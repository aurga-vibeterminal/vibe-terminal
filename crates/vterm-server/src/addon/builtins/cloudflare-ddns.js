function getFqdns(domain, subdomain) {
    if (!subdomain) return [domain];

    var fqdns = [];
    var subdomains = subdomain.split(';');
    for (var i = 0; i < subdomains.length; i++) {
        var name = subdomains[i].trim();
        if (name) fqdns.push(name + '.' + domain);
    }
    return fqdns.length > 0 ? fqdns : [domain];
}

function httpRequest(method, url, headers, body) {
    addon.log('[HTTP] ' + method + ' ' + url);
    var requestBody = body == null ? undefined : body;
    var resp = JSON.parse(addon.http_request(method, url, headers, requestBody));
    addon.log('[HTTP] Response status: ' + resp.status);
    return resp;
}

function responsePreview(body) {
    if (!body) return '';
    return body.length > 240 ? body.substring(0, 240) + '...' : body;
}

function parseCloudflareResponse(resp, action) {
    if (resp.status < 200 || resp.status >= 300) {
        var statusError = action + ' failed: HTTP ' + resp.status;
        var statusBody = responsePreview(resp.body);
        if (statusBody) statusError += ': ' + statusBody;
        return { success: false, error: statusError };
    }

    var parsed = null;
    try {
        parsed = JSON.parse(resp.body);
    } catch (error) {
        var parseError = action + ' returned invalid JSON';
        if (error && error.message) parseError += ': ' + error.message;
        var parseBody = responsePreview(resp.body);
        if (parseBody) parseError += ' (' + parseBody + ')';
        return { success: false, error: parseError };
    }

    if (!parsed.success) {
        var cfError = action + ' failed';
        if (parsed.errors && parsed.errors.length > 0) {
            var messages = [];
            for (var i = 0; i < parsed.errors.length; i++) {
                messages.push(parsed.errors[i].message || JSON.stringify(parsed.errors[i]));
            }
            cfError += ': ' + messages.join('; ');
        }
        return { success: false, error: cfError };
    }

    return { success: true, body: parsed };
}

function run() {
    addon.log('Cloudflare DDNS addon started');
    var config = addon.get_config();
    var ipv4 = addon.get_ipv4();
    var ipv6 = addon.get_ipv6();

    addon.log('Config: email=' + (config.email || '(empty)') + ', domain=' + (config.domain || '(empty)') + ', subdomain=' + (config.subdomain || '(empty)'));
    addon.log('Detected IPs: ipv4=' + (ipv4 || 'none') + ', ipv6=' + (ipv6 || 'none'));

    var email = config.email || '';
    var apiKey = config.api_key || '';
    var domain = config.domain || '';
    var subdomain = config.subdomain || '';
    var updateIpv4 = ipv4 != null && ipv4 !== '';
    var updateIpv6 = ipv6 != null && ipv6 !== '';

    if (!email || !apiKey || !domain) {
        addon.log('Missing required config: email, api_key, and domain');
        return { success: false, error: 'Missing required config: email, api_key, and domain' };
    }

    var headers = {
        'X-Auth-Email': email,
        'X-Auth-Key': apiKey,
        'Content-Type': 'application/json'
    };

    var fqdns = getFqdns(domain, subdomain);
    addon.log('Target FQDNs: ' + fqdns.join(', '));

    // Get zone ID
    var zonesResp = httpRequest('GET',
        'https://api.cloudflare.com/client/v4/zones?name=' + encodeURIComponent(domain),
        headers, null);
    var zonesParsed = parseCloudflareResponse(zonesResp, 'Zone lookup');
    if (!zonesParsed.success) {
        addon.log(zonesParsed.error);
        return { success: false, error: zonesParsed.error };
    }
    var zones = zonesParsed.body;
    if (!zones.success || !zones.result || zones.result.length === 0) {
        addon.log('Zone not found: ' + domain);
        return { success: false, error: 'Zone not found: ' + domain };
    }
    var zoneId = zones.result[0].id;
    addon.log('Found zone: ' + domain + ' (' + zoneId + ')');

    var updated = false;
    var currentIp = null;

    for (var i = 0; i < fqdns.length; i++) {
        var fqdn = fqdns[i];
        addon.log('Updating DNS records for ' + fqdn);

        // Update A record (IPv4)
        if (updateIpv4 && ipv4) {
            currentIp = ipv4;
            addon.log('Updating A record with IPv4: ' + ipv4);
            var result = updateRecord(zoneId, fqdn, 'A', ipv4, headers);
            if (!result.success) return result;
            if (result.updated) updated = true;
        } else {
            addon.log('No IPv4 address detected; deleting stale A record if present');
            var delResult = deleteRecordIfExists(zoneId, fqdn, 'A', headers);
            if (!delResult.success) return delResult;
            if (delResult.deleted) updated = true;
        }

        // Update AAAA record (IPv6)
        if (updateIpv6 && ipv6) {
            currentIp = ipv6;
            addon.log('Updating AAAA record with IPv6: ' + ipv6);
            var result = updateRecord(zoneId, fqdn, 'AAAA', ipv6, headers);
            if (!result.success) return result;
            if (result.updated) updated = true;
        } else {
            addon.log('No IPv6 address detected; deleting stale AAAA record if present');
            var delResult = deleteRecordIfExists(zoneId, fqdn, 'AAAA', headers);
            if (!delResult.success) return delResult;
            if (delResult.deleted) updated = true;
        }
    }

    addon.log('Cloudflare DDNS addon completed: success=true, ip=' + (currentIp || 'none') + ', updated=' + updated);
    return { success: true, ip: currentIp, updated: updated };
}

function deleteRecordIfExists(zoneId, fqdn, type, headers) {
    addon.log('Checking for stale ' + type + ' record: ' + fqdn);

    var lookupResp = httpRequest('GET',
        'https://api.cloudflare.com/client/v4/zones/' + zoneId +
        '/dns_records?type=' + type + '&name=' + encodeURIComponent(fqdn),
        headers, null);
    var lookupParsed = parseCloudflareResponse(lookupResp, type + ' record lookup for deletion');
    if (!lookupParsed.success) {
        addon.log(lookupParsed.error);
        return { success: false, error: lookupParsed.error };
    }
    var lookup = lookupParsed.body;

    if (lookup.result && lookup.result.length > 0) {
        var recordId = lookup.result[0].id;
        addon.log('Deleting stale ' + type + ' record for ' + fqdn + ' (id: ' + recordId + ')');
        var deleteResp = httpRequest('DELETE',
            'https://api.cloudflare.com/client/v4/zones/' + zoneId +
            '/dns_records/' + recordId,
            headers, null);
        var deleteParsed = parseCloudflareResponse(deleteResp, type + ' record delete');
        if (!deleteParsed.success) {
            addon.log(deleteParsed.error);
            return { success: false, error: deleteParsed.error };
        }
        addon.log('Deleted ' + type + ' record for ' + fqdn);
        return { success: true, deleted: true };
    }

    addon.log('No ' + type + ' record found for ' + fqdn + '; nothing to delete');
    return { success: true, deleted: false };
}

function updateRecord(zoneId, fqdn, type, ip, headers) {
    // Lookup existing record
    var lookupResp = httpRequest('GET',
        'https://api.cloudflare.com/client/v4/zones/' + zoneId +
        '/dns_records?type=' + type + '&name=' + encodeURIComponent(fqdn),
        headers, null);
    var lookupParsed = parseCloudflareResponse(lookupResp, type + ' record lookup');
    if (!lookupParsed.success) {
        addon.log(lookupParsed.error);
        return { success: false, error: lookupParsed.error };
    }
    var lookup = lookupParsed.body;

    var body = JSON.stringify({
        type: type,
        name: fqdn,
        content: ip,
        ttl: 60,
        proxied: false
    });

    if (lookup.result && lookup.result.length > 0) {
        var record = lookup.result[0];
        if (record.content === ip) {
            addon.log(type + ' record for ' + fqdn + ' is already up to date (' + ip + ')');
            return { success: true, updated: false };
        }
        addon.log(type + ' record for ' + fqdn + ' changed from ' + record.content + ' to ' + ip);
        var updateResp = httpRequest('PUT',
            'https://api.cloudflare.com/client/v4/zones/' + zoneId +
            '/dns_records/' + record.id,
            headers, body);
        var updateParsed = parseCloudflareResponse(updateResp, type + ' record update');
        if (!updateParsed.success) {
            addon.log(updateParsed.error);
            return { success: false, error: updateParsed.error };
        }
        addon.log('Updated ' + type + ' record for ' + fqdn + ' to ' + ip);
        return { success: true, updated: true };
    } else {
        addon.log(type + ' record for ' + fqdn + ' does not exist, creating...');
        var createResp = httpRequest('POST',
            'https://api.cloudflare.com/client/v4/zones/' + zoneId +
            '/dns_records',
            headers, body);
        var createParsed = parseCloudflareResponse(createResp, type + ' record create');
        if (!createParsed.success) {
            addon.log(createParsed.error);
            return { success: false, error: createParsed.error };
        }
        addon.log('Created ' + type + ' record for ' + fqdn + ' to ' + ip);
        return { success: true, updated: true };
    }
}
