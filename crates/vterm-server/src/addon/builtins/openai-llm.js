// OpenAI Compatible LLM Addon
// Implements get_default_model(), chat_completion(), stream_chat_completion()

async function get_default_model() {
  return await addon.get_config("model") || "gpt-3.5-turbo";
}

function prepareMessages(incomingMessages) {
  const systemPrompt = addon.get_config("system_prompt");
  const userPromptTemplate = addon.get_config("user_prompt");
  let messages = [];

  if (systemPrompt && !incomingMessages.some(function(m) { return m.role === "system"; })) {
    messages.push({ role: "system", content: systemPrompt });
  }

  for (var i = 0; i < incomingMessages.length; i++) {
    var msg = incomingMessages[i];
    if (msg.role === "user" && userPromptTemplate) {
      var wrapped = userPromptTemplate.replace("{text}", msg.content);
      messages.push({ role: "user", content: wrapped });
    } else {
      messages.push(msg);
    }
  }
  return messages;
}

function chat_completion(messages, options) {
  var opts = options || {};
  var baseUrl = (addon.get_config("base_url") || "https://api.openai.com/v1").replace(/\/+$/, "");
  var apiKey = addon.get_config("api_key") || "";
  var model = opts.model || addon.get_config("model") || "gpt-4";
  var temperature = opts.temperature !== undefined ? opts.temperature : parseFloat(addon.get_config("temperature") || "0.7");
  // max_input_tokens is a server-side prompt/context budget. The API's
  // max_tokens field is an output limit and uses max_output_tokens by default.
  var maxTokens = opts.max_tokens !== undefined
    ? opts.max_tokens
    : parseInt(addon.get_config("max_output_tokens") || "4096");

  var finalMessages = prepareMessages(messages);

  var headers = { "Content-Type": "application/json" };
  if (apiKey) {
    headers["Authorization"] = "Bearer " + apiKey;
  }

  var baseBody = {
    model: model,
    messages: finalMessages,
    temperature: temperature,
    max_tokens: maxTokens,
    stream: false
  };

  if (opts.tools) {
    baseBody.tools = opts.tools;
  }

  var extraBodyRaw = addon.get_config("extra_body");
  if (extraBodyRaw) {
    try {
      var extra = JSON.parse(extraBodyRaw);
      if (extra && typeof extra === "object") {
        var keys = Object.keys(extra);
        for (var i = 0; i < keys.length; i++) {
          baseBody[keys[i]] = extra[keys[i]];
        }
      }
    } catch (e) {
      // ignore invalid JSON
    }
  }

  var body = JSON.stringify(baseBody);

  var response = addon.http_request("POST", baseUrl + "/chat/completions", headers, body);
  var parsed = JSON.parse(response);

  if (parsed.status !== 200) {
    throw new Error("LLM API error: HTTP " + parsed.status + " - " + parsed.body);
  }

  return parsed.body;
}
