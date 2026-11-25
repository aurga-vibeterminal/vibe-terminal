// EasyEDA Pro MCP addon (type: "mcp").
//
// Two-tier tool surface (see docs/superpowers/specs/2026-09-28-easyeda-mcp-addon-design.md
// and 2026-09-28-easyeda-mcp-context-strategy.md):
//   Tier 1 — nine fixed tools always exposed in MCP tools/list.
//   Tier 2 — CATALOG below: a curated, categorized set of eda.* calls that the
//            agent discovers via list_categories → search_tools → describe_tool
//            and executes via easyeda_run. The catalog never leaves this file
//            unsolicited, so tools/list stays small no matter how large the
//            catalog grows.
//
// Context conventions (context-strategy spec D3):
//   - List tools paginate: {offset?, limit?} (default limit 50) and answer
//     {total, offset, limit, items}.
//   - Tools returning EDA objects note that items are live-object handles;
//     read details per handle via easyeda_call_handle.
//   - File payloads (BOM/Gerber/…) return metadata only when large.
//
// Entry points called by the Rust executor:
//   list_tools()                  → JSON string of Tier-1 descriptors
//   call_tool(name, argsJson)     → JSON string of an MCP result object
//   get_instructions()            → server instructions string (optional)
//
// Paths in this file are relative to the global `eda` object inside the EDA
// (e.g. "sch_PrimitiveComponent.getAll" resolves to eda.sch_PrimitiveComponent.getAll).
// Naming rule: d.ts class `SCH_Document` → `eda.sch_Document` (first segment lowercased).
// Entries marked [derived] follow that rule but have NOT been verified against a
// live EDA — check the official d.ts (@jlceda/pro-api-types) before relying on them.

var BRIDGE_ADDON = "easyeda";

// ---------------------------------------------------------------------------
// Result helpers
// ---------------------------------------------------------------------------

function textResult(text, isError) {
	var result = { content: [{ type: "text", text: text }] };
	if (isError) {
		result.isError = true;
	}
	return JSON.stringify(result);
}

function jsonResult(value) {
	return textResult(JSON.stringify(value, null, 2));
}

// ---------------------------------------------------------------------------
// Bridge
// ---------------------------------------------------------------------------

// Send one frame to the EDA connector and return its result value.
// Throws on transport or daemon errors.
function bridge(frame) {
	var raw = addon.bridge_request(JSON.stringify(frame));
	var resp;
	try {
		resp = JSON.parse(raw);
	} catch (e) {
		throw new Error("bridge returned invalid JSON: " + raw);
	}
	if (!resp.ok) {
		throw new Error(resp.error || "bridge request failed");
	}
	return resp.result;
}

function isBridgeUnavailable(error) {
	return String(error).indexOf("no daemon connected") !== -1;
}

// Strip a leading "eda." so callers may pass either spelling.
function normalizePath(path) {
	if (typeof path !== "string" || path.length === 0) {
		throw new Error("path must be a non-empty string, e.g. \"sch_PrimitiveComponent.getAll\"");
	}
	return path.indexOf("eda.") === 0 ? path.substring(4) : path;
}

// Convert an invoke result into MCP content. The connector serializes Blob
// results as {__eda_type:"blob", mime, base64}; everything else is JSON text.
function invokeResultContent(value) {
	if (value && value.__eda_type === "blob" && value.base64) {
		return JSON.stringify({
			content: [{ type: "image", data: value.base64, mimeType: value.mime || "image/png" }]
		});
	}
	return jsonResult(value);
}

// ---------------------------------------------------------------------------
// D3 context conventions
// ---------------------------------------------------------------------------

var DEFAULT_PAGE_LIMIT = 50;
var MAX_PAGE_LIMIT = 200;

// Page a plain array. Returns the standard pagination envelope.
function paginate(items, args) {
	var total = items.length;
	var offset = Math.max(0, Number(args.offset) || 0);
	var limit = Number(args.limit) || DEFAULT_PAGE_LIMIT;
	limit = Math.min(MAX_PAGE_LIMIT, Math.max(1, limit));
	return { total: total, offset: offset, limit: limit, items: items.slice(offset, offset + limit) };
}

// Render a connector-serialized File/Blob frame as JSON metadata. Small
// files keep their base64; large ones are withheld so the agent's context
// survives (direct-to-disk export lands with the connector update).
function filePayloadInfo(value, fallbackName) {
	if (!value || value.__eda_type !== "blob") {
		return value;
	}
	var sizeBytes = Math.floor(String(value.base64 || "").length * 3 / 4);
	var info = {
		fileName: value.name || fallbackName,
		mime: value.mime || "application/octet-stream",
		sizeBytes: sizeBytes
	};
	if (sizeBytes <= 32768) {
		info.base64 = value.base64;
	} else {
		info.note =
			"File content withheld to keep the context small. Fetch the raw bytes " +
			"with easyeda_invoke only if strictly needed.";
	}
	return info;
}

// Summary projection for component lists (D3): keep the fields an agent
// needs to find and reason about components; full:true returns raw items.
function projectComponents(items, full) {
	if (full) {
		return items;
	}
	return items.map(function (c) {
		return {
			designator: c.designator,
			name: c.name,
			footprint: c.footprint && c.footprint.name,
			componentType: c.componentType,
			x: c.x,
			y: c.y,
			primitiveId: c.primitiveId
		};
	});
}

// Primitive type → SDK class suffix, for the generic delete tools.
var SCH_PRIMITIVE_CLASSES = {
	component: "Component",
	wire: "Wire",
	text: "Text",
	rectangle: "Rectangle",
	circle: "Circle",
	arc: "Arc",
	polygon: "Polygon",
	bus: "Bus",
	pin: "Pin",
	object: "Object"
};

var PCB_PRIMITIVE_CLASSES = {
	arc: "PrimitiveArc",
	attribute: "PrimitiveAttribute",
	component: "PrimitiveComponent",
	dimension: "PrimitiveDimension",
	fill: "PrimitiveFill",
	image: "PrimitiveImage",
	line: "PrimitiveLine",
	object: "PrimitiveObject",
	pad: "PrimitivePad",
	polyline: "PrimitivePolyline",
	pour: "PrimitivePour",
	poured: "PrimitivePoured",
	region: "PrimitiveRegion",
	string: "PrimitiveString",
	via: "PrimitiveVia"
};

// Normalize a create() result: live objects arrive as handles — pluck the
// primitiveId and release the handle so agents can compose modify/delete
// without leaking references.
function createdPrimitiveInfo(value) {
	if (value && value.__eda_type === "handle") {
		var id = null;
		try {
			id = bridge({ action: "get_handle", handle: value.handle, prop: "primitiveId" });
		} catch (e) { /* property absent */ }
		try {
			bridge({ action: "release", handles: [value.handle] });
		} catch (e) { /* already gone */ }
		return { primitiveId: id };
	}
	if (value && typeof value === "object" && value.primitiveId !== undefined) {
		return { primitiveId: value.primitiveId };
	}
	return value;
}

// Geometry for the pcb_create_{pour,fill,region,polyline} tools. Two input
// forms are accepted:
//   rect   {x, y, width, height, rotation?, cornerRadius?} (mil)
//   points [[x, y], ...] (mil, >= 3 pairs)
// points are compiled to the path-command form [x0,y0,"L",x1,y1,...] which
// pcb_MathPolygon.createPolygon handles correctly. When close is true the
// first vertex is repeated at the end (pour/fill/region need closed shapes;
// polylines stay open). NEVER pass a flat [x1,y1,x2,y2,...] array to
// createPolygon — that form hangs the EDA 3.2 JS thread (verified live: the
// whole client froze and had to be restarted); on EDA 4.1+ it is rejected
// (returns undefined, verified on 4.1.60). Either way it never works.
function polygonSource(args, close) {
	var rect = args.rect;
	if (rect && typeof rect.x === "number" && typeof rect.y === "number" &&
		typeof rect.width === "number" && typeof rect.height === "number") {
		return [
			"R",
			rect.x,
			rect.y,
			rect.width,
			rect.height,
			typeof rect.rotation === "number" ? rect.rotation : 0,
			typeof rect.cornerRadius === "number" ? rect.cornerRadius : 0
		];
	}
	var points = args.points;
	if (Array.isArray(points) && points.length >= 3) {
		var src = [];
		for (var i = 0; i < points.length; i++) {
			var pt = points[i];
			if (!Array.isArray(pt) || pt.length < 2 ||
				typeof pt[0] !== "number" || typeof pt[1] !== "number") {
				throw new Error("points must be [[x, y], ...] number pairs (mil)");
			}
			if (i === 0) {
				src.push(pt[0], pt[1], "L");
			} else {
				src.push(pt[0], pt[1]);
			}
		}
		if (close) {
			src.push(points[0][0], points[0][1]);
		}
		return src;
	}
	throw new Error("pass rect {x, y, width, height} or points [[x, y], ...] (mil, >= 3 points)");
}

// Pour/fill (like pads) silently misbehave when the layer is passed as a
// name: the primitive is created and even reads the name back, but its net
// binding is dropped (verified live — net reads back null after a fresh
// getAll, while numeric-layer pours keep it). Map copper layer names to the
// numeric EPCB_LayerId values.
function copperLayerId(layer) {
	if (typeof layer === "number") {
		return layer;
	}
	var LAYERS = { TOP: 1, BOTTOM: 2, MULTI: 12 };
	var id = LAYERS[String(layer || "").toUpperCase()];
	if (typeof id !== "number") {
		throw new Error("layer must be a copper layer: TOP, BOTTOM or a numeric layer id");
	}
	return id;
}

// EDA 4.1 tightened layer parameters across the board: every
// pcb_Primitive*.create now REJECTS a layer name where 3.2 accepted it
// (pour/fill previously even dropped the net binding silently — upstream
// issue 9; on 4.1.60 the create throws instead). Map common layer names to
// the numeric EPCB_LayerId; numeric ids pass through unchanged. Accepts
// INNER1 / INNER_1 .. INNER30 and CUSTOM_n as numbers stay numeric.
function pcbLayerId(layer) {
	if (typeof layer === "number") {
		return layer;
	}
	var LAYERS = {
		TOP: 1, BOTTOM: 2,
		TOP_SILKSCREEN: 3, BOTTOM_SILKSCREEN: 4,
		TOP_SOLDER_MASK: 5, BOTTOM_SOLDER_MASK: 6,
		TOP_PASTE_MASK: 7, BOTTOM_PASTE_MASK: 8,
		TOP_ASSEMBLY: 9, BOTTOM_ASSEMBLY: 10,
		BOARD_OUTLINE: 11, MULTI: 12, DOCUMENT: 13, MECHANICAL: 14,
		RATLINE: 57, TOP_STIFFENER: 58, BOTTOM_STIFFENER: 59
	};
	var name = String(layer || "").toUpperCase().replace(/[\s-]+/g, "_");
	if (Object.prototype.hasOwnProperty.call(LAYERS, name)) {
		return LAYERS[name];
	}
	var inner = /^INNER_?(\d+)$/.exec(name);
	if (inner) {
		var n = parseInt(inner[1], 10);
		if (n >= 1 && n <= 30) {
			return 14 + n;
		}
	}
	throw new Error(
		"unknown layer '" + layer + "' — pass a numeric layer id (see pcb_list_layers) " +
		"or a name like TOP, BOTTOM, INNER1, TOP_SILKSCREEN, BOARD_OUTLINE"
	);
}

// Pagination properties shared by list-tool schemas.
var PAGE_PROPS = {
	offset: { type: "number", description: "Skip this many items (default 0)" },
	limit: { type: "number", description: "Max items to return (default 50, max 200)" }
};

var READ_ONLY = { readOnlyHint: true };

// "Argument absent" placeholder for SDK calls with optional positional
// parameters: JSON cannot express undefined, and the SDK treats a padded
// null as a real filter. The connector turns this into undefined.
var UNDEF = { __eda_type: "undef" };

// ---------------------------------------------------------------------------
// Tier 2 catalog
// ---------------------------------------------------------------------------

var CATALOG = [
	{
		category: "schematic",
		summary: "Schematic documents and primitives (components, wires, nets)",
		tools: [
			{
				name: "sch_list_components",
				summary: "List components on the current schematic (paginated summaries)",
				description:
					"Returns eda.sch_PrimitiveComponent.getAll as paginated summary objects " +
					"({designator, name, footprint, componentType, x, y, primitiveId}). " +
					"Pass full:true for the unmodified property set.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: Object.assign({ full: { type: "boolean", description: "Return raw items instead of summaries (default false)" } }, PAGE_PROPS)
				},
				invoke: {
					path: "sch_PrimitiveComponent.getAll",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					var items = Array.isArray(value) ? value : [];
					return paginate(projectComponents(items, args.full === true), args);
				}
			},
			{
				name: "sch_list_nets",
				// NOTE: eda.sch_Net.* still returns empty on EDA 4.1.60, but
				// @jlceda/pro-api-types 0.4.26 marks it "ADD since EDA v4.2" —
				// retest on v4.2 and consider switching to the official API
				// (it should also cover nets without explicit wire net labels,
				// which this wire-derived list cannot see).
				summary: "List all net names in the current schematic (paginated)",
				description:
					"Net names derived from the schematic's wires (eda.sch_PrimitiveWire.getAll) — " +
					"the eda.sch_Net summary API returns empty results on current EDA builds. " +
					"Use sch_get_net for one net's wires.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: PAGE_PROPS },
				invoke: {
					path: "sch_PrimitiveWire.getAll",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					var seen = {};
					(Array.isArray(value) ? value : []).forEach(function (w) {
						if (w && w.net) {
							seen[w.net] = true;
						}
					});
					return paginate(Object.keys(seen).sort(), args);
				}
			},
			{
				name: "sch_get_net",
				// NOTE: sch_PrimitiveWire.getAll(net) filtering is broken on EDA
				// 3.2.x–4.1.60 (always returns empty; upstream issue 4), so this
				// tool fetches all wires and filters here. Retest on EDA v4.2 —
				// if the API filter starts working, pass net through to getAll.
				summary: "All wires belonging to one schematic net (paginated)",
				description:
					"Returns the wires carrying the given net (eda.sch_PrimitiveWire.getAll " +
					"filtered by net name — the API's own net filter returns empty results " +
					"on current EDA builds, so filtering happens here).",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: Object.assign({
						net: { type: "string", description: "Net name, e.g. \"GND\"" }
					}, PAGE_PROPS),
					required: ["net"]
				},
				invoke: {
					path: "sch_PrimitiveWire.getAll",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					var wires = (Array.isArray(value) ? value : []).filter(function (w) {
						return w && w.net === args.net;
					});
					return paginate(wires, args);
				}
			},
			{
				name: "sch_cross_probe_select",
				summary: "Select/highlight components (or pins, nets) in the schematic editor",
				description:
					"Calls eda.sch_SelectControl.doCrossProbeSelect. Changes the editor's " +
					"selection/highlight state but not the design itself.",
				inputSchema: {
					type: "object",
					properties: {
						designators: {
							type: "array",
							items: { type: "string" },
							description: "Component designators, e.g. [\"R1\", \"C3\"]"
						},
						pins: {
							type: "array",
							items: { type: "string" },
							description: "Pins in \"U1_1\" format"
						},
						nets: { type: "array", items: { type: "string" } },
						highlight: { type: "boolean", description: "Also highlight (default true)" }
					}
				},
				invoke: {
					path: "sch_SelectControl.doCrossProbeSelect",
					buildArgs: function (args) {
						return [
							args.designators || UNDEF,
							args.pins || UNDEF,
							args.nets || UNDEF,
							args.highlight !== false,
							true
						];
					}
				}
			},
			{
				name: "sch_get_component",
				summary: "Full properties of one schematic component",
				description:
					"Reads eda.sch_PrimitiveComponent.get(id) in the EDA process and " +
					"returns its data properties as a plain object. Use " +
					"sch_list_components to find primitiveIds.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						id: { type: "string", description: "Primitive ID of the component" }
					},
					required: ["id"]
				},
				run: function (args) {
					if (!args.id) {
						throw new Error("id is required");
					}
					// get() returns a live object; read its direct properties
					// in-sandbox (one round trip) instead of proxying a handle.
					var code =
						"var g = await eda.sch_PrimitiveComponent.get(" + JSON.stringify(args.id) + ");" +
						"if (!g) return null;" +
						"var keys = ['primitiveId','designator','name','componentType','x','y'," +
						"'rotation','mirror','footprint','uniqueId','manufacturer','manufacturerId'," +
						"'supplier','supplierId','addIntoBom','addIntoPcb'];" +
						"var out = {};" +
						"for (var i = 0; i < keys.length; i++) { var v = g[keys[i]]; if (v !== undefined) out[keys[i]] = v; }" +
						"return out;";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "sch_get_selection",
				summary: "Currently selected schematic primitives (paginated)",
				description:
					"Returns eda.sch_SelectControl.getAllSelectedPrimitives, paginated.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: PAGE_PROPS },
				invoke: {
					path: "sch_SelectControl.getAllSelectedPrimitives",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					return paginate(Array.isArray(value) ? value : [], args);
				}
			},
			{
				name: "sch_drc_check",
				summary: "Run schematic DRC and return the errors (paginated)",
				description:
					"Calls eda.sch_Drc.check(strict, false, true) and returns the detailed " +
					"error list, paginated. Empty list means the schematic is clean.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: Object.assign({
						strict: { type: "boolean", description: "Strict check (default false)" }
					}, PAGE_PROPS)
				},
				invoke: {
					path: "sch_Drc.check",
					buildArgs: function (args) { return [args.strict === true, false, true]; }
				},
				mapResult: function (value, args) {
					return paginate(Array.isArray(value) ? value : [], args);
				}
			},
			{
				name: "sch_get_bom_file",
				summary: "Export the BOM of the current schematic (metadata; base64 when small)",
				description:
					"Calls eda.sch_ManufactureData.getBomFile. Pass filePath to write " +
					"the file straight to disk instead.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						fileType: { type: "string", description: "\"xlsx\" (default) or \"csv\"" },
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "sch_ManufactureData.getBomFile",
					buildArgs: function (args) {
						return [UNDEF, args.fileType === "csv" ? "csv" : "xlsx"];
					}
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "bom");
				}
			},
			{
				name: "sch_get_netlist_file",
				summary: "Export the netlist of the current schematic",
				description:
					"Calls eda.sch_ManufactureData.getNetlistFile. Works on EDA 4.1+ " +
					"(verified on 4.1.60). The export returns null when the document " +
					"has unsaved changes, so this tool saves the document first when " +
					"needed. Pass filePath to write the file straight to disk instead. " +
					"Note: the schematic netlist export is broken on EDA 3.2.x (always " +
					"returns nothing) — there it fails with a clear error; use " +
					"pcb_get_netlist_file or sch_list_nets connectivity instead.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				run: function (args) {
					function attempt() {
						if (typeof args.filePath === "string" && args.filePath.length > 0) {
							return bridge({
								action: "save_invoke",
								path: "sch_ManufactureData.getNetlistFile",
								args: [UNDEF, UNDEF],
								filePath: args.filePath
							});
						}
						return filePayloadInfo(
							bridge({ action: "invoke", path: "sch_ManufactureData.getNetlistFile", args: [UNDEF, UNDEF] }),
							"netlist"
						);
					}
					var value = attempt();
					if (value == null || (value && value.savedTo === null && value.sizeWarning)) {
						// Unsaved changes make the export come back null — save, retry once.
						bridge({ action: "invoke", path: "sch_Document.save", args: [] });
						value = attempt();
					}
					if (value == null) {
						throw new Error(
							"schematic netlist export returned nothing even after saving — " +
							"sch_ManufactureData.getNetlistFile is broken on EDA 3.2.x; " +
							"use pcb_get_netlist_file or sch_list_nets connectivity instead"
						);
					}
					return jsonResult(value);
				}
			},
			{
				name: "sch_get_png_file",
				summary: "Render the current schematic to a PNG image",
				description:
					"Renders the active schematic as PNG (eda.sch_ManufactureData.getPngFile " +
					"on EDA 4.1+, the getExportDocumentFile fallback on older builds). " +
					"Returns an MCP image by default; pass filePath to write the PNG " +
					"straight to disk instead.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						width: { type: "number", description: "Max width in px (4.1+ only)" },
						height: { type: "number", description: "Max height in px (4.1+ only)" },
						filePath: { type: "string", description: "Save the PNG to this local path instead of returning it" }
					}
				},
				run: function (args) {
					var hasNew = bridge({
						action: "eval",
						code: "return typeof eda.sch_ManufactureData.getPngFile === 'function';"
					});
					var path;
					var callArgs;
					if (hasNew) {
						var res;
						if (typeof args.width === "number" || typeof args.height === "number") {
							res = {};
							if (typeof args.width === "number") res.width = args.width;
							if (typeof args.height === "number") res.height = args.height;
						}
						path = "sch_ManufactureData.getPngFile";
						callArgs = [UNDEF, res || UNDEF];
					} else {
						path = "sch_ManufactureData.getExportDocumentFile";
						callArgs = [UNDEF, "PNG"];
					}
					if (typeof args.filePath === "string" && args.filePath.length > 0) {
						return jsonResult(bridge({
							action: "save_invoke",
							path: path,
							args: callArgs,
							filePath: args.filePath
						}));
					}
					return invokeResultContent(bridge({ action: "invoke", path: path, args: callArgs }));
				}
			},
			{
				name: "sch_place_component",
				summary: "Place a library component onto the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveComponent.create — modifies the design. " +
					"component: pass a result item from lib_search_devices directly, or " +
					"{libraryUuid, uuid}. Coordinates in 0.01 inch. Call doc_save afterwards.",
				inputSchema: {
					type: "object",
					properties: {
						component: { type: "object", description: "Library device/symbol reference" },
						x: { type: "number" },
						y: { type: "number" },
						subPartName: { type: "string" },
						rotation: { type: "number" },
						mirror: { type: "boolean" }
					},
					required: ["component", "x", "y"]
				},
				invoke: {
					path: "sch_PrimitiveComponent.create",
					buildArgs: function (args) {
						return [
							args.component,
							args.x,
							args.y,
							args.subPartName || UNDEF,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							typeof args.mirror === "boolean" ? args.mirror : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_wire",
				summary: "Draw a wire on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveWire.create — modifies the design. line is a " +
					"flat coordinate array [x1,y1,x2,y2,...] or an array of connected " +
					"segments; net is inferred from the endpoints when omitted. " +
					"Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						line: { type: "array", description: "Coordinates, e.g. [0,0,100,0]" },
						net: { type: "string", description: "Net name (optional)" }
					},
					required: ["line"]
				},
				invoke: {
					path: "sch_PrimitiveWire.create",
					buildArgs: function (args) {
						return [args.line, args.net || UNDEF, UNDEF, UNDEF, UNDEF];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_net_flag",
				summary: "Place a power/ground net flag on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveComponent.createNetFlag — modifies the design. " +
					"identification is one of Power, Ground, AnalogGround, ProtectGround. " +
					"Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						identification: { type: "string", description: "Power | Ground | AnalogGround | ProtectGround" },
						net: { type: "string", description: "Net name, e.g. \"VCC\" or \"GND\"" },
						x: { type: "number" },
						y: { type: "number" },
						rotation: { type: "number" },
						mirror: { type: "boolean" }
					},
					required: ["identification", "net", "x", "y"]
				},
				invoke: {
					path: "sch_PrimitiveComponent.createNetFlag",
					buildArgs: function (args) {
						return [
							args.identification,
							args.net,
							args.x,
							args.y,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							typeof args.mirror === "boolean" ? args.mirror : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_net_port",
				summary: "Place a net port on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveComponent.createNetPort — modifies the design. " +
					"direction is IN, OUT or BI. Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						direction: { type: "string", description: "IN | OUT | BI" },
						net: { type: "string" },
						x: { type: "number" },
						y: { type: "number" },
						rotation: { type: "number" },
						mirror: { type: "boolean" }
					},
					required: ["direction", "net", "x", "y"]
				},
				invoke: {
					path: "sch_PrimitiveComponent.createNetPort",
					buildArgs: function (args) {
						return [
							args.direction,
							args.net,
							args.x,
							args.y,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							typeof args.mirror === "boolean" ? args.mirror : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_net_label",
				summary: "Place a net label on the schematic (WRITE, EDA v4+ only)",
				description:
					"Calls eda.sch_PrimitiveAttribute.createNetLabel — modifies the design. " +
					"Only available on EDA v4 and newer. Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string" },
						x: { type: "number" },
						y: { type: "number" }
					},
					required: ["net", "x", "y"]
				},
				run: function (args) {
					var available = bridge({
						action: "eval",
						code: "return typeof eda.sch_PrimitiveAttribute.createNetLabel === 'function';"
					});
					if (!available) {
						throw new Error(
							"sch_create_net_label requires EDA v4 or newer — this build has no " +
							"sch_PrimitiveAttribute.createNetLabel. Use sch_create_net_flag or " +
							"sch_create_net_port instead."
						);
					}
					var result = createdPrimitiveInfo(bridge({
						action: "invoke",
						path: "sch_PrimitiveAttribute.createNetLabel",
						args: [args.x, args.y, args.net]
					}));
					// v3 builds ship createNetLabel as a stub that returns null.
					if (result === null || (result && result.primitiveId === null)) {
						throw new Error(
							"sch_create_net_label requires EDA v4 or newer — on this build " +
							"createNetLabel exists but returns null. Use sch_create_net_flag or " +
							"sch_create_net_port instead."
						);
					}
					return jsonResult(result);
				}
			},
			{
				name: "sch_modify_component",
				summary: "Modify a schematic component's properties (WRITE)",
				description:
					"Calls eda.sch_PrimitiveComponent.modify — modifies the design. Pass any " +
					"subset of {designator, name, x, y, rotation, mirror, addIntoBom, " +
					"addIntoPcb, uniqueId, manufacturer, manufacturerId, supplier, supplierId}. " +
					"Call doc_save afterwards.",
				inputSchema: {
					type: "object",
					properties: {
						id: { type: "string", description: "Primitive ID" },
						property: { type: "object", description: "Properties to change" }
					},
					required: ["id", "property"]
				},
				invoke: {
					path: "sch_PrimitiveComponent.modify",
					buildArgs: function (args) { return [args.id, args.property]; }
				}
			},
			{
				name: "sch_create_text",
				summary: "Place a text annotation on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveText.create — modifies the design. " +
					"Coordinates in 0.01 inch. alignMode: 1-9 as LEFT_TOP..RIGHT_BOTTOM " +
					"(5 = CENTER).",
				inputSchema: {
					type: "object",
					properties: {
						x: { type: "number" },
						y: { type: "number" },
						content: { type: "string" },
						rotation: { type: "number" },
						textColor: { type: "string" },
						fontName: { type: "string" },
						fontSize: { type: "number" },
						bold: { type: "boolean" },
						italic: { type: "boolean" },
						underLine: { type: "boolean" },
						alignMode: { type: "number", description: "1 LEFT_TOP … 5 CENTER … 9 RIGHT_BOTTOM" }
					},
					required: ["x", "y", "content"]
				},
				invoke: {
					path: "sch_PrimitiveText.create",
					buildArgs: function (args) {
						return [
							args.x,
							args.y,
							args.content,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							args.textColor || UNDEF,
							args.fontName || UNDEF,
							typeof args.fontSize === "number" ? args.fontSize : UNDEF,
							typeof args.bold === "boolean" ? args.bold : UNDEF,
							typeof args.italic === "boolean" ? args.italic : UNDEF,
							typeof args.underLine === "boolean" ? args.underLine : UNDEF,
							typeof args.alignMode === "number" ? args.alignMode : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_rectangle",
				summary: "Draw a rectangle on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveRectangle.create — modifies the design. " +
					"Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						topLeftX: { type: "number" },
						topLeftY: { type: "number" },
						width: { type: "number" },
						height: { type: "number" },
						cornerRadius: { type: "number" },
						rotation: { type: "number" },
						color: { type: "string" },
						fillColor: { type: "string" },
						lineWidth: { type: "number" },
						lineType: { type: "string" }
					},
					required: ["topLeftX", "topLeftY", "width", "height"]
				},
				invoke: {
					path: "sch_PrimitiveRectangle.create",
					buildArgs: function (args) {
						return [
							args.topLeftX,
							args.topLeftY,
							args.width,
							args.height,
							typeof args.cornerRadius === "number" ? args.cornerRadius : UNDEF,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							args.color || UNDEF,
							args.fillColor || UNDEF,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							args.lineType || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_circle",
				summary: "Draw a circle on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveCircle.create — modifies the design. " +
					"Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						centerX: { type: "number" },
						centerY: { type: "number" },
						radius: { type: "number" },
						color: { type: "string" },
						fillColor: { type: "string" },
						lineWidth: { type: "number" },
						lineType: { type: "string" },
						fillStyle: { type: "string" }
					},
					required: ["centerX", "centerY", "radius"]
				},
				invoke: {
					path: "sch_PrimitiveCircle.create",
					buildArgs: function (args) {
						return [
							args.centerX,
							args.centerY,
							args.radius,
							args.color || UNDEF,
							args.fillColor || UNDEF,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							args.lineType || UNDEF,
							args.fillStyle || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_arc",
				summary: "Draw an arc on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveArc.create — modifies the design. The arc " +
					"passes from (startX,startY) through the reference point to " +
					"(endX,endY). Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						startX: { type: "number" },
						startY: { type: "number" },
						referenceX: { type: "number" },
						referenceY: { type: "number" },
						endX: { type: "number" },
						endY: { type: "number" },
						color: { type: "string" },
						fillColor: { type: "string" },
						lineWidth: { type: "number" },
						lineType: { type: "string" }
					},
					required: ["startX", "startY", "referenceX", "referenceY", "endX", "endY"]
				},
				invoke: {
					path: "sch_PrimitiveArc.create",
					buildArgs: function (args) {
						return [
							args.startX,
							args.startY,
							args.referenceX,
							args.referenceY,
							args.endX,
							args.endY,
							args.color || UNDEF,
							args.fillColor || UNDEF,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							args.lineType || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_polygon",
				summary: "Draw a polygon on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitivePolygon.create — modifies the design. line is " +
					"a flat coordinate array [x1,y1,x2,y2,...]. Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						line: { type: "array", description: "Flat coordinates, e.g. [0,0,100,0,100,100]" },
						color: { type: "string" },
						fillColor: { type: "string" },
						lineWidth: { type: "number" },
						lineType: { type: "string" }
					},
					required: ["line"]
				},
				invoke: {
					path: "sch_PrimitivePolygon.create",
					buildArgs: function (args) {
						return [
							args.line,
							args.color || UNDEF,
							args.fillColor || UNDEF,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							args.lineType || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_bus",
				summary: "Draw a bus on the schematic (WRITE)",
				description:
					"Calls eda.sch_PrimitiveBus.create — modifies the design. line is a " +
					"flat coordinate array or an array of segments. Coordinates in " +
					"0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						busName: { type: "string" },
						line: { type: "array" },
						color: { type: "string" },
						lineWidth: { type: "number" },
						lineType: { type: "string" }
					},
					required: ["busName", "line"]
				},
				invoke: {
					path: "sch_PrimitiveBus.create",
					buildArgs: function (args) {
						return [
							args.busName,
							args.line,
							args.color || UNDEF,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							args.lineType || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_create_pin",
				summary: "Add a pin to a symbol in the symbol editor (WRITE)",
				description:
					"Calls eda.sch_PrimitivePin.create — modifies the design. Used when " +
					"editing a library symbol document (see lib_open_in_editor). " +
					"pinShape: None | Inverted | Clock | Inverted Clock. pinType: IN | " +
					"OUT | BI | Passive | Power | Ground | … Coordinates in 0.01 inch.",
				inputSchema: {
					type: "object",
					properties: {
						x: { type: "number" },
						y: { type: "number" },
						pinNumber: { type: "string" },
						pinName: { type: "string" },
						rotation: { type: "number" },
						pinLength: { type: "number" },
						pinColor: { type: "string" },
						pinShape: { type: "string" },
						pinType: { type: "string" }
					},
					required: ["x", "y", "pinNumber"]
				},
				invoke: {
					path: "sch_PrimitivePin.create",
					buildArgs: function (args) {
						return [
							args.x,
							args.y,
							args.pinNumber,
							args.pinName || UNDEF,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							typeof args.pinLength === "number" ? args.pinLength : UNDEF,
							args.pinColor || UNDEF,
							args.pinShape || UNDEF,
							args.pinType || UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "sch_delete_primitives",
				summary: "Delete schematic primitives by ID (DESTRUCTIVE)",
				description:
					"Deletes primitives — modifies the design irreversibly until the next " +
					"undo. type is one of component, wire, text, rectangle, circle, arc, " +
					"polygon, bus, pin, object.",
				annotations: { destructiveHint: true },
				inputSchema: {
					type: "object",
					properties: {
						type: { type: "string" },
						ids: { type: "array", items: { type: "string" } }
					},
					required: ["type", "ids"]
				},
				run: function (args) {
					var cls = SCH_PRIMITIVE_CLASSES[String(args.type || "").toLowerCase()];
					if (!cls) {
						throw new Error("unknown primitive type '" + args.type + "'; expected one of: " + Object.keys(SCH_PRIMITIVE_CLASSES).join(", "));
					}
					if (!Array.isArray(args.ids) || args.ids.length === 0) {
						throw new Error("ids must be a non-empty array");
					}
					return jsonResult(bridge({
						action: "invoke",
						path: "sch_Primitive" + cls + ".delete",
						args: [args.ids]
					}));
				}
			}
		]
	},
	{
		category: "pcb",
		summary: "PCB documents, primitives, and manufacturing outputs",
		tools: [
			{
				name: "pcb_list_components",
				summary: "List components on the current PCB (paginated summaries)",
				description:
					"Returns eda.pcb_PrimitiveComponent.getAll as paginated summary objects " +
					"({designator, name, footprint, componentType, x, y, primitiveId}). " +
					"Requires a PCB document to be active. Pass full:true for raw items.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: Object.assign({ full: { type: "boolean", description: "Return raw items instead of summaries (default false)" } }, PAGE_PROPS)
				},
				invoke: {
					path: "pcb_PrimitiveComponent.getAll",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					var items = Array.isArray(value) ? value : [];
					return paginate(projectComponents(items, args.full === true), args);
				}
			},
			{
				name: "pcb_get_bom_file",
				summary: "Export the BOM of the current PCB (metadata; base64 when small)",
				description:
					"Calls eda.pcb_ManufactureData.getBomFile. Returns the file name, size " +
					"and MIME; base64 content is included only for small files. " +
					"Pass filePath to write the file straight to disk instead " +
					"(returns {savedTo, sizeBytes} — analyze with file tools).",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						fileType: { type: "string", description: "\"xlsx\" (default) or \"csv\"" },
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "pcb_ManufactureData.getBomFile",
					buildArgs: function (args) {
						return [UNDEF, args.fileType === "csv" ? "csv" : "xlsx"];
					}
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "bom");
				}
			},
			{
				name: "pcb_get_netlist_file",
				summary: "Export the netlist of the current PCB (metadata; base64 when small)",
				description:
					"Calls eda.pcb_ManufactureData.getNetlistFile. Returns the file name, " +
					"size and MIME; base64 content is included only for small files. " +
					"Pass filePath to write the file straight to disk instead.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "pcb_ManufactureData.getNetlistFile",
					buildArgs: function () { return []; }
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "netlist");
				}
			},
			{
				name: "pcb_get_gerber_file",
				summary: "Export Gerber files of the current PCB (metadata only, usually large)",
				description:
					"Calls eda.pcb_ManufactureData.getGerberFile. Gerber archives are " +
					"large, so this usually returns metadata without content. " +
					"Pass filePath to write the archive straight to disk instead. " +
					"This can be slow — consider raising the addon timeout.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "pcb_ManufactureData.getGerberFile",
					buildArgs: function () { return []; }
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "gerber");
				}
			},
			{
				name: "pcb_list_nets",
				summary: "List all net names on the current PCB (paginated)",
				description: "Returns eda.pcb_Net.getAllNetsName, paginated.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: PAGE_PROPS },
				invoke: {
					path: "pcb_Net.getAllNetsName",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					return paginate(Array.isArray(value) ? value : [], args);
				}
			},
			{
				name: "pcb_get_net",
				summary: "Full details of one PCB net (length, color, …)",
				description: "Returns eda.pcb_Net.getNet(net).",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string", description: "Net name" }
					},
					required: ["net"]
				},
				invoke: {
					path: "pcb_Net.getNet",
					buildArgs: function (args) { return [args.net]; }
				}
			},
			{
				name: "pcb_list_layers",
				summary: "List all layers of the current PCB",
				description: "Returns eda.pcb_Layer.getAllLayers with per-layer details.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "pcb_Layer.getAllLayers",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "pcb_get_selection",
				summary: "Currently selected PCB primitives (paginated)",
				description:
					"Returns eda.pcb_SelectControl.getAllSelectedPrimitives, paginated.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: PAGE_PROPS },
				invoke: {
					path: "pcb_SelectControl.getAllSelectedPrimitives",
					buildArgs: function () { return []; }
				},
				mapResult: function (value, args) {
					return paginate(Array.isArray(value) ? value : [], args);
				}
			},
			{
				name: "pcb_drc_check",
				summary: "Run PCB DRC and return the errors (paginated)",
				description:
					"Calls eda.pcb_Drc.check(strict, false, true) and returns the detailed " +
					"error list, paginated. Empty list means the PCB is clean. Can be slow " +
					"on large boards.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: Object.assign({
						strict: { type: "boolean", description: "Strict check (default false)" }
					}, PAGE_PROPS)
				},
				invoke: {
					path: "pcb_Drc.check",
					buildArgs: function (args) { return [args.strict === true, false, true]; }
				},
				mapResult: function (value, args) {
					return paginate(Array.isArray(value) ? value : [], args);
				}
			},
			{
				name: "pcb_get_pickandplace_file",
				summary: "Export the pick-and-place (coordinate) file of the current PCB",
				description:
					"Calls eda.pcb_ManufactureData.getPickAndPlaceFile. Pass filePath to " +
					"write the file straight to disk instead.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						fileType: { type: "string", description: "\"xlsx\" (default) or \"csv\"" },
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "pcb_ManufactureData.getPickAndPlaceFile",
					buildArgs: function (args) {
						return [UNDEF, args.fileType === "csv" ? "csv" : "xlsx"];
					}
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "pickandplace");
				}
			},
			{
				name: "pcb_get_pdf_file",
				summary: "Export the current PCB as a PDF document",
				description:
					"Calls eda.pcb_ManufactureData.getPdfFile. Pass filePath to write " +
					"the file straight to disk instead. Can be slow.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				invoke: {
					path: "pcb_ManufactureData.getPdfFile",
					buildArgs: function () { return [UNDEF]; }
				},
				mapResult: function (value) {
					return filePayloadInfo(value, "pcb-pdf");
				}
			},
			{
				name: "pcb_import_changes",
				summary: "Sync schematic changes into the PCB (WRITE, opens a modal dialog)",
				description:
					"Calls eda.pcb_Document.importChanges — updates the PCB to match the " +
					"schematic. WARNING: the EDA shows a modal \"confirm importing " +
					"changes\" dialog that a human must confirm or cancel; while it is " +
					"open, other design operations stall or fail. Warn the user before " +
					"calling this. Can be slow on large designs.",
				annotations: { destructiveHint: true },
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "pcb_Document.importChanges",
					buildArgs: function () { return [UNDEF]; }
				}
			},
			{
				name: "pcb_place_component",
				summary: "Place a library component onto the PCB (WRITE)",
				description:
					"Calls eda.pcb_PrimitiveComponent.create — modifies the design. " +
					"component: pass a result item from lib_search_devices or " +
					"{libraryUuid, uuid}. layer is TOP or BOTTOM. Coordinates in mil. " +
					"Call doc_save afterwards.",
				inputSchema: {
					type: "object",
					properties: {
						component: { type: "object" },
						layer: { type: "string", description: "TOP | BOTTOM" },
						x: { type: "number" },
						y: { type: "number" },
						rotation: { type: "number" }
					},
					required: ["component", "layer", "x", "y"]
				},
				invoke: {
					path: "pcb_PrimitiveComponent.create",
					buildArgs: function (args) {
						return [
							args.component,
							copperLayerId(args.layer),
							args.x,
							args.y,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_track",
				summary: "Route a straight track segment on the PCB (WRITE)",
				description:
					"Calls eda.pcb_PrimitiveLine.create — modifies the design. layer is a " +
					"copper layer name (e.g. TOP, BOTTOM, INNER1). Coordinates in mil.",
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string" },
						layer: { type: "string" },
						startX: { type: "number" },
						startY: { type: "number" },
						endX: { type: "number" },
						endY: { type: "number" },
						lineWidth: { type: "number" }
					},
					required: ["net", "layer", "startX", "startY", "endX", "endY"]
				},
				invoke: {
					path: "pcb_PrimitiveLine.create",
					buildArgs: function (args) {
						return [
							args.net,
							pcbLayerId(args.layer),
							args.startX,
							args.startY,
							args.endX,
							args.endY,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_via",
				summary: "Place a via on the PCB (WRITE)",
				description:
					"Calls eda.pcb_PrimitiveVia.create — modifies the design. Diameters in mil.",
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string" },
						x: { type: "number" },
						y: { type: "number" },
						holeDiameter: { type: "number" },
						diameter: { type: "number" }
					},
					required: ["net", "x", "y", "holeDiameter", "diameter"]
				},
				invoke: {
					path: "pcb_PrimitiveVia.create",
					buildArgs: function (args) {
						return [args.net, args.x, args.y, args.holeDiameter, args.diameter, UNDEF, UNDEF, UNDEF, UNDEF];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_pour",
				summary: "Create a copper pour region on the PCB (WRITE)",
				description:
					"Builds the region in the EDA process and calls " +
					"eda.pcb_PrimitivePour.create, then rebuildCopperRegion() to " +
					"generate the actual copper — modifies the design. Geometry: " +
					"rect {x, y, width, height, rotation?, cornerRadius?} or " +
					"points [[x, y], ...] (mil, >= 3 points; closed automatically). " +
					"layer is a copper layer (\"TOP\"/\"BOTTOM\" or numeric id — " +
					"names are mapped internally because the SDK drops the net " +
					"binding when a layer name is passed). " +
					"fillMethod: solid | 90grid | 45grid (aliases grid / grid45). " +
					"poured in the result reports whether copper was generated — " +
					"false on a board with no pads/tracks on that net is normal.",
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string" },
						layer: { type: "string", description: "Copper layer, e.g. \"TOP\"" },
						rect: { type: "object", description: "{x, y, width, height, rotation?, cornerRadius?} in mil" },
						points: { type: "array", items: { type: "array", items: { type: "number" } }, description: "[[x, y], ...] polygon vertices in mil (>= 3)" },
						pourName: { type: "string" },
						fillMethod: { type: "string", description: "solid | 90grid | 45grid (default solid)" },
						priority: { type: "number" },
						lineWidth: { type: "number", description: "mil" }
					},
					required: ["net", "layer"]
				},
				run: function (args) {
					var source = polygonSource(args, true);
					var layerId = copperLayerId(args.layer);
					var fillMap = { solid: "solid", grid: "90grid", "90grid": "90grid", grid45: "45grid", "45grid": "45grid" };
					var fill = "solid";
					if (typeof args.fillMethod === "string" && args.fillMethod !== "") {
						fill = fillMap[args.fillMethod];
						if (!fill) {
							throw new Error("fillMethod must be solid | 90grid | 45grid (aliases grid / grid45)");
						}
					}
					// Do everything in one in-sandbox eval: cross-call handle
					// chaining (createPolygon → createPour) is unreliable here.
					var createArgs = [args.net, layerId, "__POLY__", fill, null,
						args.pourName || null,
						typeof args.priority === "number" ? args.priority : null,
						typeof args.lineWidth === "number" ? args.lineWidth : null];
					var code =
						"var p = eda.pcb_MathPolygon.createPolygon(" + JSON.stringify(source) + ");" +
						"if (!p) { throw new Error('createPolygon failed — points must form a valid polygon'); }" +
						"var a = " + JSON.stringify(createArgs) + ";" +
						"a[2] = p;" +
						"a = a.map(function (v) { return v === null ? undefined : v; });" +
						"var pour = await eda.pcb_PrimitivePour.create.apply(eda.pcb_PrimitivePour, a);" +
						"if (!pour) { return null; }" +
						"var id = pour.getState_PrimitiveId ? pour.getState_PrimitiveId() : (pour.primitiveId !== undefined ? pour.primitiveId : null);" +
						"var poured = false;" +
						"try { poured = !!(await pour.rebuildCopperRegion()); } catch (e) {}" +
						"return { primitiveId: id, poured: poured };";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "pcb_create_string",
				summary: "Place a text string (e.g. silkscreen label) on the PCB (WRITE)",
				description:
					"Calls eda.pcb_PrimitiveString.create — modifies the design. layer is " +
					"typically a silkscreen layer (\"TOP_SILKSCREEN\" / \"BOTTOM_SILKSCREEN\" " +
					"or a layer id from pcb_list_layers). x, y, fontSize and lineWidth in " +
					"mil. alignMode: 1-9 as LEFT_TOP..RIGHT_BOTTOM (5 = CENTER).",
				inputSchema: {
					type: "object",
					properties: {
						layer: { type: "string", description: "e.g. \"TOP_SILKSCREEN\"" },
						x: { type: "number" },
						y: { type: "number" },
						text: { type: "string" },
						fontFamily: { type: "string", description: "Default \"Arial\"" },
						fontSize: { type: "number", description: "mil, default 50" },
						lineWidth: { type: "number", description: "mil, default 6" },
						alignMode: { type: "number", description: "1 LEFT_TOP … 5 CENTER … 9 RIGHT_BOTTOM (default 5)" },
						rotation: { type: "number" },
						reverse: { type: "boolean" },
						mirror: { type: "boolean" },
						primitiveLock: { type: "boolean" }
					},
					required: ["layer", "x", "y", "text"]
				},
				invoke: {
					path: "pcb_PrimitiveString.create",
					buildArgs: function (args) {
						return [
							pcbLayerId(args.layer),
							args.x,
							args.y,
							args.text,
							args.fontFamily || "Arial",
							typeof args.fontSize === "number" ? args.fontSize : 50,
							typeof args.lineWidth === "number" ? args.lineWidth : 6,
							typeof args.alignMode === "number" ? args.alignMode : 5,
							typeof args.rotation === "number" ? args.rotation : 0,
							args.reverse === true,
							0,
							args.mirror === true,
							args.primitiveLock === true
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_pad",
				summary: "Create a pad on the PCB or in a footprint (WRITE)",
				description:
					"Calls eda.pcb_PrimitivePad.create — modifies the design. layer is " +
					"\"TOP\" (1), \"BOTTOM\" (2) or \"MULTI\" (12, through-hole) — " +
					"unlike tracks, pads require the numeric layer id. pad is a " +
					"shape tuple: [\"ELLIPSE\", w, h], [\"OVAL\", w, h], " +
					"[\"NGON\", w, h] or [\"RECT\", w, h, cornerRadius]. hole " +
					"(optional): [\"ROUND\", diameter] or [\"SLOT\", length, width]. " +
					"Coordinates in mil.",
				inputSchema: {
					type: "object",
					properties: {
						layer: { type: "string", description: "TOP | BOTTOM | MULTI" },
						padNumber: { type: "string" },
						x: { type: "number" },
						y: { type: "number" },
						rotation: { type: "number" },
						pad: { type: "array", description: "Shape tuple, e.g. [\"RECT\", 60, 40, 0]" },
						net: { type: "string" },
						hole: { type: "array", description: "e.g. [\"ROUND\", 32]" }
					},
					required: ["layer", "padNumber", "x", "y"]
				},
				invoke: {
					path: "pcb_PrimitivePad.create",
					buildArgs: function (args) {
						// Pads (unlike tracks/strings) reject layer names — the SDK
						// wants the numeric EPCB_LayerId: TOP=1, BOTTOM=2, MULTI=12.
						var LAYERS = { TOP: 1, BOTTOM: 2, MULTI: 12 };
						var layer = typeof args.layer === "number" ? args.layer : LAYERS[String(args.layer || "").toUpperCase()];
						if (typeof layer !== "number") {
							throw new Error("layer must be TOP, BOTTOM or MULTI");
						}
						return [
							layer,
							args.padNumber,
							args.x,
							args.y,
							typeof args.rotation === "number" ? args.rotation : UNDEF,
							Array.isArray(args.pad) ? args.pad : UNDEF,
							args.net || UNDEF,
							Array.isArray(args.hole) ? args.hole : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_arc",
				summary: "Draw an arc track/outline on the PCB (WRITE)",
				description:
					"Calls eda.pcb_PrimitiveArc.create — modifies the design. net may be " +
					"empty for non-electrical layers. Coordinates in mil.",
				inputSchema: {
					type: "object",
					properties: {
						net: { type: "string" },
						layer: { type: "string", description: "e.g. \"TOP\"" },
						startX: { type: "number" },
						startY: { type: "number" },
						endX: { type: "number" },
						endY: { type: "number" },
						arcAngle: { type: "number", description: "Arc angle in degrees" },
						lineWidth: { type: "number" },
						primitiveLock: { type: "boolean" }
					},
					required: ["layer", "startX", "startY", "endX", "endY", "arcAngle"]
				},
				invoke: {
					path: "pcb_PrimitiveArc.create",
					buildArgs: function (args) {
						return [
							args.net || "",
							pcbLayerId(args.layer),
							args.startX,
							args.startY,
							args.endX,
							args.endY,
							args.arcAngle,
							typeof args.lineWidth === "number" ? args.lineWidth : UNDEF,
							UNDEF,
							args.primitiveLock === true ? true : UNDEF
						];
					}
				},
				mapResult: createdPrimitiveInfo
			},
			{
				name: "pcb_create_fill",
				summary: "Create a solid fill region on the PCB (WRITE)",
				description:
					"Builds the shape in the EDA process and calls " +
					"eda.pcb_PrimitiveFill.create — modifies the design. Geometry: " +
					"rect {x, y, width, height, rotation?, cornerRadius?} or " +
					"points [[x, y], ...] (mil, >= 3 points; closed automatically). " +
					"layer: \"TOP\"/\"BOTTOM\" or numeric id (names are mapped " +
					"internally — the SDK drops the net binding on layer names). " +
					"fillMode: 0 SOLID, 1 MESH, 2 INNER_ELECTRICAL_LAYER.",
				inputSchema: {
					type: "object",
					properties: {
						layer: { type: "string", description: "e.g. \"TOP\"" },
						rect: { type: "object", description: "{x, y, width, height, rotation?, cornerRadius?} in mil" },
						points: { type: "array", items: { type: "array", items: { type: "number" } }, description: "[[x, y], ...] polygon vertices in mil (>= 3)" },
						net: { type: "string" },
						fillMode: { type: "number" },
						lineWidth: { type: "number" },
						primitiveLock: { type: "boolean" }
					},
					required: ["layer"]
				},
				run: function (args) {
					var source = polygonSource(args, true);
					var code =
						"var p = eda.pcb_MathPolygon.createPolygon(" + JSON.stringify(source) + ");" +
						"var f = await eda.pcb_PrimitiveFill.create(" + JSON.stringify(copperLayerId(args.layer)) + ", p, " +
						(args.net ? JSON.stringify(args.net) : "undefined") + ", " +
						(typeof args.fillMode === "number" ? args.fillMode : "undefined") + ", " +
						(typeof args.lineWidth === "number" ? args.lineWidth : "undefined") + ", " +
						(args.primitiveLock === true) + ");" +
						"return f ? { primitiveId: f.primitiveId !== undefined ? f.primitiveId : null } : null;";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "pcb_create_region",
				summary: "Create a rule region (keepout etc.) on the PCB (WRITE)",
				description:
					"Builds the shape in the EDA process and calls " +
					"eda.pcb_PrimitiveRegion.create — modifies the design. Geometry: " +
					"rect {x, y, width, height} or points [[x, y], ...] (mil, " +
					">= 3 points; closed automatically). layer is a copper layer " +
					"or \"MULTI\". ruleTypes " +
					"subset of: 2 NO_COMPONENTS, 5 NO_WIRES, 6 NO_FILLS, 7 NO_POURS, " +
					"8 NO_INNER_ELECTRICAL_LAYERS, 9 FOLLOW_REGION_RULE.",
				inputSchema: {
					type: "object",
					properties: {
						layer: { type: "string", description: "Copper layer or \"MULTI\"" },
						rect: { type: "object" },
						points: { type: "array", items: { type: "array", items: { type: "number" } }, description: "[[x, y], ...] polygon vertices in mil (>= 3)" },
						ruleTypes: { type: "array", items: { type: "number" } },
						regionName: { type: "string" },
						lineWidth: { type: "number" },
						primitiveLock: { type: "boolean" }
					},
					required: ["layer"]
				},
				run: function (args) {
					var source = polygonSource(args, true);
					var code =
						"var p = eda.pcb_MathPolygon.createPolygon(" + JSON.stringify(source) + ");" +
						"var r = await eda.pcb_PrimitiveRegion.create(" + JSON.stringify(pcbLayerId(args.layer)) + ", p, " +
						(Array.isArray(args.ruleTypes) ? JSON.stringify(args.ruleTypes) : "undefined") + ", " +
						(args.regionName ? JSON.stringify(args.regionName) : "undefined") + ", " +
						(typeof args.lineWidth === "number" ? args.lineWidth : "undefined") + ", " +
						(args.primitiveLock === true) + ");" +
						"return r ? { primitiveId: r.primitiveId !== undefined ? r.primitiveId : null } : null;";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "pcb_create_polyline",
				summary: "Draw a polyline (outline/detail) on the PCB (WRITE)",
				description:
					"Builds the shape in the EDA process and calls " +
					"eda.pcb_PrimitivePolyline.create — modifies the design. Geometry: " +
					"rect {x, y, width, height} or points [[x, y], ...] (mil, " +
					">= 3 points; left open — pass close:true to close the loop). " +
					"net may be empty for non-electrical layers.",
				inputSchema: {
					type: "object",
					properties: {
						layer: { type: "string", description: "e.g. \"TOP\" or \"BOARD_OUTLINE\"" },
						rect: { type: "object" },
						points: { type: "array", items: { type: "array", items: { type: "number" } }, description: "[[x, y], ...] vertices in mil (>= 3)" },
						close: { type: "boolean", description: "Close the loop by repeating the first point (default false)" },
						net: { type: "string" },
						lineWidth: { type: "number" },
						primitiveLock: { type: "boolean" }
					},
					required: ["layer"]
				},
				run: function (args) {
					var source = polygonSource(args, args.close === true);
					var code =
						"var p = eda.pcb_MathPolygon.createPolygon(" + JSON.stringify(source) + ");" +
						"var l = await eda.pcb_PrimitivePolyline.create(" + JSON.stringify(args.net || "") + ", " +
						JSON.stringify(pcbLayerId(args.layer)) + ", p, " +
						(typeof args.lineWidth === "number" ? args.lineWidth : "undefined") + ", " +
						(args.primitiveLock === true) + ");" +
						"return l ? { primitiveId: l.primitiveId !== undefined ? l.primitiveId : null } : null;";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "pcb_get_3d_file",
				summary: "Export the board 3D model (STEP/OBJ)",
				description:
					"Calls eda.pcb_ManufactureData.get3DFile. fileType is \"step\" or " +
					"\"obj\". element filters the contents, subset of \"Component Model\", " +
					"\"Via\", \"Silkscreen\", \"Wire In Signal Layer\". modelMode is " +
					"\"Outfit\" or \"Parts\". Files are large — pass filePath to write " +
					"straight to disk instead of returning content.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						fileName: { type: "string" },
						fileType: { type: "string", description: "step | obj" },
						element: { type: "array", items: { type: "string" } },
						modelMode: { type: "string", description: "Outfit | Parts" },
						autoGenerateModels: { type: "boolean" },
						filePath: { type: "string", description: "Save the export to this local path instead of returning content" }
					}
				},
				run: function (args) {
					var callArgs = [
						args.fileName || UNDEF,
						args.fileType || UNDEF,
						Array.isArray(args.element) ? args.element : UNDEF,
						args.modelMode || UNDEF,
						typeof args.autoGenerateModels === "boolean" ? args.autoGenerateModels : UNDEF
					];
					if (typeof args.filePath === "string" && args.filePath.length > 0) {
						return jsonResult(bridge({
							action: "save_invoke",
							path: "pcb_ManufactureData.get3DFile",
							args: callArgs,
							filePath: args.filePath
						}));
					}
					return filePayloadInfo(
						bridge({ action: "invoke", path: "pcb_ManufactureData.get3DFile", args: callArgs }),
						"3d model"
					);
				}
			},
			{
				name: "pcb_delete_primitives",
				summary: "Delete PCB primitives by ID (DESTRUCTIVE)",
				description:
					"Deletes primitives — modifies the design irreversibly until the next " +
					"undo. type is one of arc, attribute, component, dimension, fill, " +
					"image, line, object, pad, polyline, pour, poured, region, string, via.",
				annotations: { destructiveHint: true },
				inputSchema: {
					type: "object",
					properties: {
						type: { type: "string" },
						ids: { type: "array", items: { type: "string" } }
					},
					required: ["type", "ids"]
				},
				run: function (args) {
					var cls = PCB_PRIMITIVE_CLASSES[String(args.type || "").toLowerCase()];
					if (!cls) {
						throw new Error("unknown primitive type '" + args.type + "'; expected one of: " + Object.keys(PCB_PRIMITIVE_CLASSES).join(", "));
					}
					if (!Array.isArray(args.ids) || args.ids.length === 0) {
						throw new Error("ids must be a non-empty array");
					}
					return jsonResult(bridge({
						action: "invoke",
						path: "pcb_" + cls + ".delete",
						args: [args.ids]
					}));
				}
			},
			{
				name: "pcb_auto_routing",
				summary: "Auto-route the PCB (WRITE, slow, beta)",
				description:
					"Calls eda.pcb_Document.autoRouting — routes unrouted nets and modifies " +
					"the design extensively. Slow on large boards; consider raising the " +
					"addon timeout. Beta API.",
				annotations: { destructiveHint: true },
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "pcb_Document.autoRouting",
					buildArgs: function () { return [UNDEF]; }
				}
			}
		]
	},
	{
		category: "library",
		summary: "Component/footprint library lookup and previews",
		tools: [
			{
				name: "lib_search_devices",
				summary: "Search the component library by keyword (paginated)",
				description:
					"Calls eda.lib_Device.search with a keyword, paginated. " +
					"Use limit/page to walk large result sets. For exact matches by " +
					"LCSC C-number use lib_get_devices_by_lcsc_ids; the device " +
					"properties include a \"Datasheet\" URL you can download with " +
					"your own HTTP tools.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						keyword: { type: "string", description: "Search keyword, e.g. \"STM32F103\"" },
						limit: { type: "number", description: "Results per page (default 50)" },
						page: { type: "number", description: "Page number (default 1)" }
					},
					required: ["keyword"]
				},
				invoke: {
					path: "lib_Device.search",
					buildArgs: function (args) {
						return [
							args.keyword,
							UNDEF,
							UNDEF,
							UNDEF,
							Number(args.limit) || DEFAULT_PAGE_LIMIT,
							Number(args.page) || 1
						];
					}
				}
			},
			{
				name: "lib_footprint_render_image",
				summary: "Render a library footprint to an image",
				description:
					"Calls eda.lib_Footprint.getRenderImage with {footprintUuid, libraryUuid}. " +
					"Returns an MCP image.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						uuid: { type: "string", description: "Footprint UUID" },
						libraryUuid: { type: "string", description: "Library containing the footprint" }
					},
					required: ["uuid", "libraryUuid"]
				},
				invoke: {
					path: "lib_Footprint.getRenderImage",
					buildArgs: function (args) {
						return [{ footprintUuid: args.uuid, libraryUuid: args.libraryUuid }];
					}
				}
			},
			{
				name: "lib_get_device",
				summary: "Full properties of one library device",
				description:
					"Returns eda.lib_Device.get(deviceUuid, libraryUuid?). The " +
					"property.otherProperty map carries the supply-chain fields, " +
					"including \"Supplier Part\" (LCSC C-number) and \"Datasheet\" — a " +
					"URL like https://item.szlcsc.com/datasheet/<part>.pdf. To get the " +
					"datasheet itself, read that URL and download it with your own " +
					"HTTP/file tools; the EDA SDK has no datasheet-download API.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						uuid: { type: "string", description: "Device UUID" },
						libraryUuid: { type: "string", description: "Library UUID (optional)" }
					},
					required: ["uuid"]
				},
				invoke: {
					path: "lib_Device.get",
					buildArgs: function (args) { return [args.uuid, args.libraryUuid || UNDEF]; }
				}
			},
			{
				name: "lib_get_devices_by_lcsc_ids",
				summary: "Look up library devices by LCSC part number(s) (C-numbers)",
				description:
					"Calls eda.lib_Device.getByLcscIds. lcscIds is one C-number " +
					"(\"C15127\") or an array of them. Set allowMultiMatch to return " +
					"every match instead of the first per id. As with lib_get_device, " +
					"the result properties include the \"Datasheet\" URL — download " +
					"it with your own HTTP tools (no SDK datasheet API).",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						lcscIds: { description: "C-number string (\"C15127\") or array of strings" },
						libraryUuid: { type: "string", description: "Restrict to this library (optional)" },
						allowMultiMatch: { type: "boolean", description: "Return all matches per id (default false)" }
					},
					required: ["lcscIds"]
				},
				invoke: {
					path: "lib_Device.getByLcscIds",
					buildArgs: function (args) {
						return [
							args.lcscIds,
							args.libraryUuid || UNDEF,
							typeof args.allowMultiMatch === "boolean" ? args.allowMultiMatch : UNDEF
						];
					}
				}
			},
			{
				name: "lib_search_devices_by_properties",
				summary: "Precise device search by property values (paginated)",
				description:
					"Calls eda.lib_Device.searchByProperties. Pass any subset of " +
					"{name, value, symbolName, footprintName, supplierId, partNumber, manufacturer, manufacturerId}.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						properties: { type: "object", description: "Property filter, e.g. {footprintName: \"0603\"}" },
						limit: { type: "number", description: "Results per page (default 50)" },
						page: { type: "number", description: "Page number (default 1)" }
					},
					required: ["properties"]
				},
				invoke: {
					path: "lib_Device.searchByProperties",
					buildArgs: function (args) {
						return [
							args.properties,
							UNDEF,
							UNDEF,
							UNDEF,
							Number(args.limit) || DEFAULT_PAGE_LIMIT,
							Number(args.page) || 1
						];
					}
				}
			},
			{
				name: "lib_get_device_by_lcsc_id",
				summary: "Look up a device by its LCSC part number (e.g. C25804)",
				description:
					"Calls eda.lib_Device.getByLcscIds with one C-number. " +
					"Not available on private deployments.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						lcscId: { type: "string", description: "LCSC part number, e.g. \"C25804\"" }
					},
					required: ["lcscId"]
				},
				invoke: {
					path: "lib_Device.getByLcscIds",
					buildArgs: function (args) { return [args.lcscId]; }
				}
			},
			{
				name: "lib_search_symbols",
				summary: "Search the symbol library by keyword (paginated)",
				description: "Calls eda.lib_Symbol.search with a keyword, paginated.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						keyword: { type: "string" },
						limit: { type: "number", description: "Results per page (default 50)" },
						page: { type: "number", description: "Page number (default 1)" }
					},
					required: ["keyword"]
				},
				invoke: {
					path: "lib_Symbol.search",
					buildArgs: function (args) {
						return [
							args.keyword,
							UNDEF,
							UNDEF,
							UNDEF,
							Number(args.limit) || DEFAULT_PAGE_LIMIT,
							Number(args.page) || 1
						];
					}
				}
			},
			{
				name: "lib_search_footprints",
				summary: "Search the footprint library by keyword (paginated)",
				description: "Calls eda.lib_Footprint.search with a keyword, paginated.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						keyword: { type: "string" },
						limit: { type: "number", description: "Results per page (default 50)" },
						page: { type: "number", description: "Page number (default 1)" }
					},
					required: ["keyword"]
				},
				invoke: {
					path: "lib_Footprint.search",
					buildArgs: function (args) {
						return [
							args.keyword,
							UNDEF,
							UNDEF,
							Number(args.limit) || DEFAULT_PAGE_LIMIT,
							Number(args.page) || 1
						];
					}
				}
			},
			{
				name: "lib_symbol_render_image",
				summary: "Render a library symbol to an image",
				description:
					"Calls eda.lib_Symbol.getRenderImage with {symbolUuid, libraryUuid, subPartName?}. " +
					"Returns an MCP image.",
				annotations: READ_ONLY,
				inputSchema: {
					type: "object",
					properties: {
						uuid: { type: "string", description: "Symbol UUID" },
						libraryUuid: { type: "string", description: "Library containing the symbol" },
						subPartName: { type: "string", description: "Sub-part for multi-part symbols (optional)" }
					},
					required: ["uuid", "libraryUuid"]
				},
				invoke: {
					path: "lib_Symbol.getRenderImage",
					buildArgs: function (args) {
						return [{
							symbolUuid: args.uuid,
							libraryUuid: args.libraryUuid,
							subPartName: args.subPartName || UNDEF
						}];
					}
				}
			},
			{
				name: "lib_list_libraries",
				summary: "List the system/personal/project/favorite library UUIDs",
				description:
					"Returns the UUIDs of the built-in libraries from " +
					"eda.lib_LibrariesList — the personal library UUID is the target " +
					"for self-built symbols/footprints/devices.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				run: function () {
					var code =
						"var L = eda.lib_LibrariesList;" +
						"return {" +
						"system: await L.getSystemLibraryUuid()," +
						"personal: await L.getPersonalLibraryUuid()," +
						"project: await L.getProjectLibraryUuid()," +
						"favorite: await L.getFavoriteLibraryUuid()" +
						"};";
					return jsonResult(bridge({ action: "eval", code: code }));
				}
			},
			{
				name: "lib_create_classification",
				summary: "Create a library classification (primary or secondary) (WRITE)",
				description:
					"Calls eda.lib_Classification.createPrimary/createSecondary. " +
					"libraryType: DEVICE | SYMBOL | FOOTPRINT | CBB | MODEL. Pass " +
					"primaryUuid to create a secondary classification under it.",
				inputSchema: {
					type: "object",
					properties: {
						libraryUuid: { type: "string" },
						libraryType: { type: "string", description: "DEVICE | SYMBOL | FOOTPRINT | CBB | MODEL" },
						name: { type: "string", description: "Classification name" },
						primaryUuid: { type: "string", description: "Parent primary classification UUID (for secondary)" }
					},
					required: ["libraryUuid", "libraryType", "name"]
				},
				run: function (args) {
					var LIB_TYPES = { DEVICE: "3", SYMBOL: "2", FOOTPRINT: "4", CBB: "1", MODEL: "5" };
					var t = LIB_TYPES[String(args.libraryType || "").toUpperCase()];
					if (!t) {
						throw new Error("libraryType must be one of: " + Object.keys(LIB_TYPES).join(", "));
					}
					if (args.primaryUuid) {
						return jsonResult(bridge({
							action: "invoke",
							path: "lib_Classification.createSecondary",
							args: [args.libraryUuid, t, args.primaryUuid, args.name]
						}));
					}
					return jsonResult(bridge({
						action: "invoke",
						path: "lib_Classification.createPrimary",
						args: [args.libraryUuid, t, args.name]
					}));
				}
			},
			{
				name: "lib_create_symbol",
				summary: "Create an empty symbol in a library (WRITE)",
				description:
					"Calls eda.lib_Symbol.create and returns the new symbol UUID. " +
					"Workflow: create → lib_open_in_editor → draw with sch_create_* " +
					"tools (sch_create_pin for pins) → lib_update_document_source. " +
					"symbolType: 2 COMPONENT (default), 18 NET_FLAG, 19 NET_PORT, " +
					"20 DRAWING, 21 NON_ELECTRICAL.",
				inputSchema: {
					type: "object",
					properties: {
						libraryUuid: { type: "string", description: "Target library (e.g. personal library from lib_list_libraries)" },
						name: { type: "string" },
						classification: { description: "ILIB_ClassificationIndex object or string array (optional)" },
						symbolType: { type: "number" },
						description: { type: "string" }
					},
					required: ["libraryUuid", "name"]
				},
				invoke: {
					path: "lib_Symbol.create",
					buildArgs: function (args) {
						return [
							args.libraryUuid,
							args.name,
							args.classification !== undefined ? args.classification : UNDEF,
							typeof args.symbolType === "number" ? args.symbolType : UNDEF,
							args.description || UNDEF,
							UNDEF
						];
					}
				}
			},
			{
				name: "lib_create_footprint",
				summary: "Create an empty footprint in a library (WRITE)",
				description:
					"Calls eda.lib_Footprint.create and returns the new footprint UUID. " +
					"Workflow: create → lib_open_in_editor → draw with pcb_create_* " +
					"tools (pcb_create_pad for pads) → lib_update_document_source.",
				inputSchema: {
					type: "object",
					properties: {
						libraryUuid: { type: "string" },
						name: { type: "string" },
						classification: { description: "ILIB_ClassificationIndex object or string array (optional)" },
						description: { type: "string" }
					},
					required: ["libraryUuid", "name"]
				},
				invoke: {
					path: "lib_Footprint.create",
					buildArgs: function (args) {
						return [
							args.libraryUuid,
							args.name,
							args.classification !== undefined ? args.classification : UNDEF,
							args.description || UNDEF,
							UNDEF
						];
					}
				}
			},
			{
				name: "lib_create_device",
				summary: "Create a device linking symbol + footprint (+3D model) (WRITE)",
				description:
					"Calls eda.lib_Device.create and returns the new device UUID. " +
					"association links the parts: {symbol: {uuid, libraryUuid}, " +
					"footprint: {uuid, libraryUuid}, model3D: {uuid, libraryUuid}}.",
				inputSchema: {
					type: "object",
					properties: {
						libraryUuid: { type: "string" },
						name: { type: "string" },
						classification: { description: "ILIB_ClassificationIndex object or string array (optional)" },
						association: { type: "object", description: "{symbol?, footprint?, model3D?} each {uuid, libraryUuid}" },
						description: { type: "string" }
					},
					required: ["libraryUuid", "name"]
				},
				invoke: {
					path: "lib_Device.create",
					buildArgs: function (args) {
						return [
							args.libraryUuid,
							args.name,
							args.classification !== undefined ? args.classification : UNDEF,
							args.association !== undefined ? args.association : UNDEF,
							args.description || UNDEF,
							UNDEF
						];
					}
				}
			},
			{
				name: "lib_update_document_source",
				summary: "Persist edits to a library symbol/footprint document (WRITE)",
				description:
					"Calls eda.lib_Symbol.updateDocumentSource or the footprint " +
					"equivalent. documentSource is the serialized document as returned " +
					"by sys_get_document_source while the library document is active.",
				inputSchema: {
					type: "object",
					properties: {
						kind: { type: "string", description: "symbol | footprint" },
						uuid: { type: "string" },
						libraryUuid: { type: "string" },
						documentSource: { type: "string" }
					},
					required: ["kind", "uuid", "libraryUuid", "documentSource"]
				},
				run: function (args) {
					var kind = String(args.kind || "").toLowerCase();
					if (kind !== "symbol" && kind !== "footprint") {
						throw new Error("kind must be 'symbol' or 'footprint'");
					}
					var cls = kind === "footprint" ? "lib_Footprint" : "lib_Symbol";
					return jsonResult(bridge({
						action: "invoke",
						path: cls + ".updateDocumentSource",
						args: [args.uuid, args.libraryUuid, args.documentSource]
					}));
				}
			},
			{
				name: "lib_open_in_editor",
				summary: "Open a library symbol/footprint in the editor",
				description:
					"Calls eda.lib_Symbol.openInEditor or the footprint equivalent, " +
					"making the library document active so sch_create_*/pcb_create_* " +
					"tools can draw into it.",
				inputSchema: {
					type: "object",
					properties: {
						kind: { type: "string", description: "symbol | footprint" },
						uuid: { type: "string" },
						libraryUuid: { type: "string" }
					},
					required: ["kind", "uuid", "libraryUuid"]
				},
				run: function (args) {
					var kind = String(args.kind || "").toLowerCase();
					if (kind !== "symbol" && kind !== "footprint") {
						throw new Error("kind must be 'symbol' or 'footprint'");
					}
					var cls = kind === "footprint" ? "lib_Footprint" : "lib_Symbol";
					return jsonResult(bridge({
						action: "invoke",
						path: cls + ".openInEditor",
						args: [args.uuid, args.libraryUuid]
					}));
				}
			},
			{
				name: "lib_delete_item",
				summary: "Delete a library item (DESTRUCTIVE)",
				description:
					"Deletes a device, symbol, footprint, 3D model or CBB from a " +
					"library — irreversible. kind: device | symbol | footprint | " +
					"model3d | cbb.",
				annotations: { destructiveHint: true },
				inputSchema: {
					type: "object",
					properties: {
						kind: { type: "string" },
						uuid: { type: "string" },
						libraryUuid: { type: "string" }
					},
					required: ["kind", "uuid", "libraryUuid"]
				},
				run: function (args) {
					var CLASSES = {
						device: "lib_Device",
						symbol: "lib_Symbol",
						footprint: "lib_Footprint",
						model3d: "lib_3DModel",
						cbb: "lib_Cbb"
					};
					var cls = CLASSES[String(args.kind || "").toLowerCase()];
					if (!cls) {
						throw new Error("kind must be one of: " + Object.keys(CLASSES).join(", "));
					}
					return jsonResult(bridge({
						action: "invoke",
						path: cls + ".delete",
						args: [args.uuid, args.libraryUuid]
					}));
				}
			}
		]
	},
	{
		category: "document",
		summary: "Active document/window state and canvas capture",
		tools: [
			{
				name: "doc_get_current_document_info",
				summary: "Info about the currently active document",
				description:
					"Calls eda.dmt_SelectControl.getCurrentDocumentInfo — document type, " +
					"UUID, tab id and owning project/library UUIDs.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "dmt_SelectControl.getCurrentDocumentInfo",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "doc_canvas_screenshot",
				summary: "Capture the current canvas as an image",
				description:
					"Calls eda.dmt_EditorControl.getCurrentRenderedAreaImage. " +
					"Returns an MCP image.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "dmt_EditorControl.getCurrentRenderedAreaImage",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "doc_get_current_project_info",
				summary: "Details of the currently open project",
				description:
					"Calls eda.dmt_Project.getCurrentProjectInfo — project name, UUID, " +
					"collaboration mode and more.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "dmt_Project.getCurrentProjectInfo",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "doc_list_schematics",
				summary: "List schematics (and their pages) in the open project",
				description: "Calls eda.dmt_Schematic.getAllSchematicsInfo.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "dmt_Schematic.getAllSchematicsInfo",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "doc_list_pcbs",
				summary: "List PCBs in the open project",
				description: "Calls eda.dmt_Pcb.getAllPcbsInfo.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "dmt_Pcb.getAllPcbsInfo",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "doc_open_document",
				summary: "Open a document (schematic page / PCB / panel) in the editor",
				description:
					"Calls eda.dmt_EditorControl.openDocument(uuid) and activates the tab. " +
					"Use doc_list_schematics / doc_list_pcbs to find UUIDs. Many sch_*/pcb_* " +
					"tools operate on the active document, so open the right one first.",
				inputSchema: {
					type: "object",
					properties: {
						uuid: { type: "string", description: "Document UUID" }
					},
					required: ["uuid"]
				},
				invoke: {
					path: "dmt_EditorControl.openDocument",
					buildArgs: function (args) { return [args.uuid]; }
				}
			},
			{
				name: "doc_save",
				summary: "Save the active document (schematic or PCB)",
				description:
					"Detects the active document type and calls the matching save " +
					"(eda.sch_Document.save or eda.pcb_Document.save).",
				inputSchema: { type: "object", properties: {} },
				run: function () {
					var info = bridge({
						action: "invoke",
						path: "dmt_SelectControl.getCurrentDocumentInfo",
						args: []
					});
					if (!info || (info.documentType !== 1 && info.documentType !== 3)) {
						throw new Error(
							"no schematic or PCB document is active (documentType " +
								(info && info.documentType) + ")"
						);
					}
					var path = info.documentType === 1 ? "sch_Document.save" : "pcb_Document.save";
					return jsonResult({
						saved: bridge({ action: "invoke", path: path, args: [] }),
						via: path
					});
				}
			}
		]
	},
	{
		category: "system",
		summary: "EDA environment information and OS file dialogs",
		tools: [
			{
				name: "sys_get_version",
				summary: "EasyEDA Pro editor version string",
				description: "Calls eda.sys_Environment.getEditorCurrentVersion.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "sys_Environment.getEditorCurrentVersion",
					buildArgs: function () { return [true]; }
				}
			},
			{
				name: "sys_open_read_file_dialog",
				summary: "Open a native file-open dialog and return the chosen file",
				description:
					"Calls eda.sys_FileSystem.openReadFileDialog. Blocks until the user " +
					"picks or cancels — consider raising the addon timeout. Requires the " +
					"user to be present at the machine.",
				inputSchema: { type: "object", properties: {} },
				invoke: {
					path: "sys_FileSystem.openReadFileDialog",
					buildArgs: function () { return []; }
				}
			},
			{
				name: "sys_get_environment_info",
				summary: "EDA environment summary (version, client mode, online state, user)",
				description:
					"Composite of eda.sys_Environment calls: editor version, client/web, " +
					"online/offline mode and the logged-in user.",
				annotations: READ_ONLY,
				inputSchema: { type: "object", properties: {} },
				run: function () {
					function q(path, args) {
						return bridge({ action: "invoke", path: path, args: args || [] });
					}
					var user = q("sys_Environment.getUserInfo") || {};
					return jsonResult({
						version: q("sys_Environment.getEditorCurrentVersion", [true]),
						isClient: q("sys_Environment.isClient"),
						isOnlineMode: q("sys_Environment.isOnlineMode"),
						isOfflineMode: q("sys_Environment.isOfflineMode"),
						user: { username: user.username, nickname: user.nickname }
					});
				}
			},
			{
				name: "sys_get_document_source",
				summary: "Read the active document's full source (large — use filePath)",
				description:
					"Calls eda.sys_FileManager.getDocumentSource. The source is in EasyEDA " +
					"Pro document-source (eprj3 record) format — field semantics are " +
					"documented in the easyeda-format / easyeda-eprj3 agent skills. The " +
					"source is large, so responses over 32 KB are withheld with a preview; " +
					"pass filePath to write the whole source straight to disk and analyze " +
					"with file tools.",
				annotations: READ_ONLY,
				saveInvoke: true,
				inputSchema: {
					type: "object",
					properties: {
						filePath: { type: "string", description: "Save the source to this local path instead of returning it" }
					}
				},
				invoke: {
					path: "sys_FileManager.getDocumentSource",
					buildArgs: function () { return []; }
				},
				mapResult: function (value) {
					var text = typeof value === "string" ? value : JSON.stringify(value);
					if (text.length <= 32768) {
						return value;
					}
					return {
						sizeBytes: text.length,
						preview: text.substring(0, 2048),
						note: "Document source withheld to keep the context small. " +
							"Re-run with filePath to save it to disk and analyze with file tools."
					};
				}
			}
		]
	}
];

function findCatalogTool(name) {
	for (var c = 0; c < CATALOG.length; c++) {
		var tools = CATALOG[c].tools;
		for (var t = 0; t < tools.length; t++) {
			if (tools[t].name === name) {
				return { category: CATALOG[c].category, tool: tools[t] };
			}
		}
	}
	return null;
}

// ---------------------------------------------------------------------------
// Tier 1 tools
// ---------------------------------------------------------------------------

var TIER1_TOOLS = [
	{
		name: "easyeda_health",
		description:
			"Check whether the EasyEDA connector is connected to the bridge and the EDA responds to ping.",
		annotations: READ_ONLY,
		inputSchema: { type: "object", properties: {} }
	},
	{
		name: "easyeda_invoke",
		description:
			"Universal escape hatch: call any eda.* API by path. " +
			"Example: {path: \"sch_PrimitiveComponent.getAll\", args: []}. " +
			"The path resolves from the global eda object; a leading \"eda.\" prefix is optional. " +
			"Can perform any operation the SDK allows, including destructive ones.",
		annotations: { openWorldHint: true },
		inputSchema: {
			type: "object",
			properties: {
				path: { type: "string", description: "Dotted path under the global eda object" },
				args: {
					type: "array",
					description: "Positional arguments for the call (default [])"
				}
			},
			required: ["path"]
		}
	},
	{
		name: "easyeda_call_handle",
		description:
			"Call a method on a live EDA object previously returned as a handle " +
			"({__eda_type:\"handle\", handle:\"h-N\"}).",
		annotations: { openWorldHint: true },
		inputSchema: {
			type: "object",
			properties: {
				handle: { type: "string" },
				method: { type: "string" },
				args: { type: "array" }
			},
			required: ["handle", "method"]
		}
	},
	{
		name: "easyeda_release",
		description: "Release live-object handles so the EDA connector can drop its references.",
		annotations: { idempotentHint: true },
		inputSchema: {
			type: "object",
			properties: {
				handles: { type: "array", items: { type: "string" } }
			},
			required: ["handles"]
		}
	},
	{
		name: "easyeda_screenshot",
		description:
			"Capture the current EDA canvas as an MCP image. " +
			"Pass scale < 1 to downscale (cheaper context).",
		annotations: READ_ONLY,
		inputSchema: {
			type: "object",
			properties: {
				scale: { type: "number", description: "Image scale, 0-1 (default 1)" }
			}
		}
	},
	{
		name: "easyeda_screenshot_region",
		description:
			"Capture a region of the canvas: zooms to the region, screenshots, " +
			"then restores the previous view. Coordinates in canvas units.",
		inputSchema: {
			type: "object",
			properties: {
				left: { type: "number" },
				right: { type: "number" },
				top: { type: "number" },
				bottom: { type: "number" },
				scale: { type: "number", description: "Image scale, 0-1 (default 1)" }
			},
			required: ["left", "right", "top", "bottom"]
		}
	},
	{
		name: "easyeda_batch",
		description:
			"Run several eda.* invoke calls in one bridge round trip. " +
			"Example: {calls: [{path: \"sch_Net.getAllNetsName\"}, {path: \"sys_Environment.isClient\"}]}. " +
			"Returns an array of per-call {ok, result|error} in the same order.",
		annotations: { openWorldHint: true },
		inputSchema: {
			type: "object",
			properties: {
				calls: {
					type: "array",
					items: {
						type: "object",
						properties: {
							path: { type: "string" },
							args: { type: "array" }
						},
						required: ["path"]
					},
					description: "Up to 50 invoke calls"
				},
				stopOnError: { type: "boolean", description: "Abort the batch on the first failure (default false)" }
			},
			required: ["calls"]
		}
	},
	{
		name: "easyeda_eval",
		description:
			"Execute JavaScript inside the EDA sandbox with full access to the global eda object. " +
			"The code runs as an async function body with `eda` in scope; its return value is " +
			"the result. Prefer this for multi-step read analysis — filtering and aggregation " +
			"happen in the EDA process and only the final answer crosses over. " +
			"Can perform any operation the SDK allows, including destructive ones.",
		annotations: { openWorldHint: true },
		inputSchema: {
			type: "object",
			properties: {
				code: { type: "string", description: "Async function body, e.g. \"return await eda.sys_Environment.isClient();\"" }
			},
			required: ["code"]
		}
	},
	{
		name: "easyeda_list_categories",
		description:
			"List catalog categories with tool counts and one-line summaries. " +
			"Start here to discover catalog tools.",
		annotations: READ_ONLY,
		inputSchema: { type: "object", properties: {} }
	},
	{
		name: "easyeda_search_tools",
		description:
			"Search the catalog by keyword (optionally within one category). " +
			"Returns matching tool names with one-line summaries — no schemas.",
		annotations: READ_ONLY,
		inputSchema: {
			type: "object",
			properties: {
				query: { type: "string" },
				category: { type: "string" }
			},
			required: ["query"]
		}
	},
	{
		name: "easyeda_describe_tool",
		description:
			"Get the full description and inputSchema of one catalog tool before running it.",
		annotations: READ_ONLY,
		inputSchema: {
			type: "object",
			properties: {
				name: { type: "string" }
			},
			required: ["name"]
		}
	},
	{
		name: "easyeda_run",
		description:
			"Execute a catalog tool by name. Use easyeda_describe_tool first to learn its arguments.",
		annotations: { openWorldHint: true },
		inputSchema: {
			type: "object",
			properties: {
				name: { type: "string" },
				args: { type: "object" }
			},
			required: ["name"]
		}
	}
];

function toolHealth() {
	try {
		var pong = bridge({ action: "ping" });
		return jsonResult({ connected: true, pong: pong });
	} catch (e) {
		if (isBridgeUnavailable(e)) {
			return jsonResult({
				connected: false,
				hint: "Start EasyEDA Pro with the connector extension (or mock-daemon.mjs for testing)."
			});
		}
		throw e;
	}
}

function toolInvoke(args) {
	var result = bridge({
		action: "invoke",
		path: normalizePath(args.path),
		args: Array.isArray(args.args) ? args.args : []
	});
	return invokeResultContent(result);
}

function toolCallHandle(args) {
	if (!args.handle || !args.method) {
		throw new Error("handle and method are required");
	}
	var result = bridge({
		action: "call_handle",
		handle: args.handle,
		method: args.method,
		args: Array.isArray(args.args) ? args.args : []
	});
	return invokeResultContent(result);
}

function toolRelease(args) {
	if (!Array.isArray(args.handles) || args.handles.length === 0) {
		throw new Error("handles must be a non-empty array");
	}
	bridge({ action: "release", handles: args.handles });
	return textResult("released " + args.handles.join(", "));
}

function blobContent(result) {
	if (!result || result.__eda_type !== "blob") {
		return jsonResult(result);
	}
	return JSON.stringify({
		content: [{ type: "image", data: result.base64, mimeType: result.mime || "image/png" }]
	});
}

function toolScreenshot(args) {
	return blobContent(bridge({
		action: "screenshot",
		scale: typeof args.scale === "number" ? args.scale : 1
	}));
}

function toolScreenshotRegion(args) {
	for (var i = 0; i < 4; i++) {
		if (typeof args[["left", "right", "top", "bottom"][i]] !== "number") {
			throw new Error("left, right, top and bottom are required numbers");
		}
	}
	bridge({
		action: "invoke",
		path: "dmt_EditorControl.zoomToRegion",
		args: [args.left, args.right, args.top, args.bottom]
	});
	var shot;
	try {
		shot = bridge({
			action: "screenshot",
			scale: typeof args.scale === "number" ? args.scale : 1
		});
	} finally {
		bridge({ action: "invoke", path: "dmt_EditorControl.zoomToAllPrimitives", args: [] });
	}
	return blobContent(shot);
}

function toolBatch(args) {
	if (!Array.isArray(args.calls) || args.calls.length === 0) {
		throw new Error("calls must be a non-empty array");
	}
	if (args.calls.length > 50) {
		throw new Error("at most 50 calls per batch");
	}
	var calls = args.calls.map(function (c, i) {
		if (!c || typeof c.path !== "string" || c.path.length === 0) {
			throw new Error("calls[" + i + "] needs a path");
		}
		return { action: "invoke", path: normalizePath(c.path), args: Array.isArray(c.args) ? c.args : [] };
	});
	var result = bridge({ action: "batch", calls: calls, stopOnError: args.stopOnError === true });
	return jsonResult(result.results);
}

function toolEval(args) {
	if (typeof args.code !== "string" || args.code.trim().length === 0) {
		throw new Error("code must be a non-empty string");
	}
	var result = bridge({ action: "eval", code: args.code });
	return invokeResultContent(result);
}

function toolListCategories() {
	var categories = CATALOG.map(function (c) {
		return { category: c.category, tools: c.tools.length, summary: c.summary };
	});
	return jsonResult(categories);
}

function toolSearchTools(args) {
	var query = String(args.query || "").toLowerCase();
	var matches = [];
	CATALOG.forEach(function (c) {
		if (args.category && c.category !== args.category) {
			return;
		}
		c.tools.forEach(function (t) {
			var haystack = (t.name + " " + t.summary).toLowerCase();
			if (haystack.indexOf(query) !== -1) {
				matches.push({ name: t.name, category: c.category, summary: t.summary });
			}
		});
	});
	return jsonResult(matches);
}

function toolDescribeTool(args) {
	var found = findCatalogTool(args.name);
	if (!found) {
		throw new Error("unknown catalog tool: " + args.name);
	}
	return jsonResult({
		name: found.tool.name,
		category: found.category,
		description: found.tool.description,
		inputSchema: found.tool.inputSchema,
		path: found.tool.invoke ? "eda." + found.tool.invoke.path : "(composite)"
	});
}

function toolRun(args) {
	var found = findCatalogTool(args.name);
	if (!found) {
		throw new Error("unknown catalog tool: " + args.name);
	}
	var toolArgs = args.args || {};
	// Composite tools implement run() directly (multiple bridge calls).
	if (found.tool.run) {
		return found.tool.run(toolArgs);
	}
	var callArgs = found.tool.invoke.buildArgs(toolArgs);
	// File egress (context-strategy D6): route the export straight to disk
	// when the caller gives a filePath.
	if (found.tool.saveInvoke && typeof toolArgs.filePath === "string" && toolArgs.filePath.length > 0) {
		return jsonResult(bridge({
			action: "save_invoke",
			path: found.tool.invoke.path,
			args: callArgs,
			filePath: toolArgs.filePath
		}));
	}
	var result = bridge({
		action: "invoke",
		path: found.tool.invoke.path,
		args: callArgs
	});
	if (found.tool.mapResult) {
		return jsonResult(found.tool.mapResult(result, toolArgs));
	}
	return invokeResultContent(result);
}

// ---------------------------------------------------------------------------
// Entry points (called by the Rust executor)
// ---------------------------------------------------------------------------

function list_tools() {
	return JSON.stringify(TIER1_TOOLS);
}

// Server instructions surfaced by the MCP host at initialize time
// (context-strategy spec D1/D8).
function get_instructions() {
	return [
		"EasyEDA Pro via the vterm bridge. Workflow:",
		"1. Discover tools: easyeda_list_categories, then easyeda_search_tools(query),",
		"   then easyeda_describe_tool(name) for the schema, then easyeda_run(name, args).",
		"2. List tools are paginated ({offset, limit}, default 50) and answer",
		"   {total, offset, limit, items}. Page instead of fetching everything.",
		"3. easyeda_eval runs JavaScript inside the EDA process — prefer it for",
		"   multi-step read analysis so only the final answer crosses over. Use",
		"   easyeda_batch for a handful of independent reads in one round trip.",
		"4. EDA objects arrive as live-object handles; read them with",
		"   easyeda_call_handle and release them with easyeda_release when done.",
		"5. File tools (BOM/Gerber/netlist) accept filePath: the export is then",
		"   written straight to disk and you get {savedTo, sizeBytes}. Analyze",
		"   saved files with file tools (grep, ranged reads) — never pull large",
		"   payloads into the conversation via easyeda_invoke or easyeda_eval.",
		"6. easyeda_invoke, easyeda_call_handle and easyeda_eval are unrestricted",
		"   escape hatches — prefer catalog tools, and never place orders",
		"   (place*Order*) or reset global settings. When calling",
		"   pcb_MathPolygon.createPolygon directly, NEVER pass a flat point array",
		"   [x1,y1,x2,y2,...] — on EDA 3.2 that form freezes the whole client;",
		"   on 4.1+ it is rejected. The path-command form [x0,y0,\"L\",x1,y1,...]",
		"   is safe; the pcb_create_* shape tools build it from points [[x,y],...]",
		"   for you",
		"7. Screenshots are expensive: pass scale < 1 and use",
		"   easyeda_screenshot_region instead of repeated full-canvas captures.",
		"8. For heavy multi-step analysis, prefer a subagent so raw EDA",
		"   payloads stay out of the main conversation.",
		"9. Related agent skills (install client-side if available):",
		"   - easyeda-api: full official API reference (120+ classes) — consult",
		"     it before writing large easyeda_eval scripts. Its own WebSocket",
		"     bridge overlaps with this MCP; when both are present, use this MCP.",
		"   - easyeda-format / easyeda-eprj3: document-source (eprj3 record)",
		"     format specs — needed to interpret sys_get_document_source output",
		"     or to author .eprj3 project files offline (that workflow does not",
		"     go through this MCP at all)."
	].join("\n");
}

function call_tool(name, argsJson) {
	var args;
	try {
		args = JSON.parse(argsJson || "{}");
	} catch (e) {
		return textResult("invalid tool arguments JSON: " + e, true);
	}
	try {
		switch (name) {
			case "easyeda_health": return toolHealth();
			case "easyeda_invoke": return toolInvoke(args);
			case "easyeda_call_handle": return toolCallHandle(args);
			case "easyeda_release": return toolRelease(args);
			case "easyeda_screenshot": return toolScreenshot(args);
			case "easyeda_screenshot_region": return toolScreenshotRegion(args);
			case "easyeda_batch": return toolBatch(args);
			case "easyeda_eval": return toolEval(args);
			case "easyeda_list_categories": return toolListCategories();
			case "easyeda_search_tools": return toolSearchTools(args);
			case "easyeda_describe_tool": return toolDescribeTool(args);
			case "easyeda_run": return toolRun(args);
			default:
				return textResult("unknown tool: " + name, true);
		}
	} catch (e) {
		return textResult(String((e && e.message) || e), true);
	}
}
