use serde::{Deserialize, Serialize};

use crate::config::types::{
    ConfigError, EnvironmentGroup, EnvironmentNode, EnvironmentVariable, TerminalServerConfig,
};

// ---------------------------------------------------------------------------
// Server-managed MCP bearer-token leaf
// ---------------------------------------------------------------------------

/// Fixed node id of the environment leaf the server generates for the MCP
/// access token. Non-numeric so it never clashes with user node ids (which
/// are always numeric and allocated by `next_node_id`).
pub(super) const MCP_TOKEN_LEAF_ID: &str = "mcp-token";
/// Display name of the generated leaf, shown in the webui's environment
/// tree. Users cannot rename it — the leaf is fully server-managed.
pub(super) const MCP_TOKEN_LEAF_NAME: &str = "Vibe Terminal MCP Bearer Token";
/// Environment variable key carrying the MCP access token. Agent templates
/// reference it via `--bearer-token-env-var VTERM_MCP_TOKEN`; users apply the
/// leaf to a terminal before running the setup command.
pub(super) const MCP_TOKEN_VARIABLE_KEY: &str = "VTERM_MCP_TOKEN";

/// Is this node id the server-managed MCP token leaf? Such nodes are rejected
/// by every user mutation endpoint (rename/move/duplicate/delete/variables).
pub(super) fn is_mcp_token_leaf(node_id: &str) -> bool {
    node_id == MCP_TOKEN_LEAF_ID
}

/// Keep the MCP token leaf in step with the MCP config: while MCP is enabled
/// the leaf exists under the root carrying `VTERM_MCP_TOKEN = access_token`
/// (created or refreshed as needed); when MCP is disabled it is removed.
/// Returns whether the tree changed, so callers can skip a pointless persist.
pub(super) fn sync_mcp_token_leaf(config: &mut TerminalServerConfig) -> bool {
    if !config.mcp.enabled {
        return config.environment.root.remove_node(MCP_TOKEN_LEAF_ID);
    }
    let desired = vec![EnvironmentVariable {
        key: MCP_TOKEN_VARIABLE_KEY.to_string(),
        value: config.mcp.access_token.clone(),
    }];
    match config.environment.root.find_node_mut(MCP_TOKEN_LEAF_ID) {
        Some(EnvironmentNode::Leaf { variables, .. }) => {
            if *variables == desired {
                return false;
            }
            *variables = desired;
            true
        }
        // Something else holds the reserved id (hand-edited config file):
        // replace it with the managed leaf.
        Some(_) => {
            config.environment.root.remove_node(MCP_TOKEN_LEAF_ID);
            insert_mcp_token_leaf(&mut config.environment.root, desired);
            true
        }
        None => {
            insert_mcp_token_leaf(&mut config.environment.root, desired);
            true
        }
    }
}

/// Insert the managed leaf at the front of the root's children so it is the
/// first thing users see in the environment tree.
fn insert_mcp_token_leaf(root: &mut EnvironmentGroup, variables: Vec<EnvironmentVariable>) {
    let node = EnvironmentNode::Leaf {
        id: MCP_TOKEN_LEAF_ID.to_string(),
        name: MCP_TOKEN_LEAF_NAME.to_string(),
        variables,
    };
    // Inserting at index 0 under the root always succeeds
    // (`insert_into_children` accepts any index <= children.len()).
    assert!(root.insert_node(None, node, Some(0)));
}

// ---------------------------------------------------------------------------
// Web DTOs (camelCase mirror of the config tree)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebEnvironmentVariable {
    pub key: String,
    pub value: String,
}

/// Tagged union mirroring `EnvironmentNode`. The `kind` discriminator is
/// serialised as `"group"` / `"leaf"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(super) enum WebEnvironmentNode {
    Group {
        id: String,
        name: String,
        #[serde(default)]
        children: Vec<WebEnvironmentNode>,
    },
    Leaf {
        id: String,
        name: String,
        #[serde(default)]
        variables: Vec<WebEnvironmentVariable>,
        /// True for server-managed leaves (the MCP bearer token) that the
        /// webui must display but not let users edit.
        #[serde(default)]
        readonly: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebEnvironmentConfig {
    pub root: WebEnvironmentGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebEnvironmentGroup {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub children: Vec<WebEnvironmentNode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum WebNodeKind {
    Group,
    Leaf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebCreateNodeRequest {
    /// Parent group id. When `None` the new node is attached directly under the
    /// root (a top-level node).
    #[serde(default)]
    pub parent_id: Option<String>,
    pub kind: WebNodeKind,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebRenameNodeRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebUpdateLeafRequest {
    pub variables: Vec<WebEnvironmentVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebMoveNodeRequest {
    /// Destination parent group id. `None` moves the node to the root level.
    #[serde(default)]
    pub target_parent_id: Option<String>,
    /// Insert position (0-based) among the destination's children. `None`
    /// appends to the end.
    #[serde(default)]
    pub index: Option<usize>,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

pub(super) fn environment_config_to_web(config: &TerminalServerConfig) -> WebEnvironmentConfig {
    WebEnvironmentConfig {
        root: web_environment_group(&config.environment.root),
    }
}

pub(super) fn web_environment_group(group: &EnvironmentGroup) -> WebEnvironmentGroup {
    WebEnvironmentGroup {
        id: group.id.clone(),
        name: group.name.clone(),
        children: group.children.iter().map(web_environment_node).collect(),
    }
}

pub(super) fn web_environment_node(node: &EnvironmentNode) -> WebEnvironmentNode {
    match node {
        EnvironmentNode::Group { id, name, children } => WebEnvironmentNode::Group {
            id: id.clone(),
            name: name.clone(),
            children: children.iter().map(web_environment_node).collect(),
        },
        EnvironmentNode::Leaf {
            id,
            name,
            variables,
        } => WebEnvironmentNode::Leaf {
            id: id.clone(),
            name: name.clone(),
            variables: variables
                .iter()
                .map(|v| WebEnvironmentVariable {
                    key: v.key.clone(),
                    value: v.value.clone(),
                })
                .collect(),
            readonly: is_mcp_token_leaf(id),
        },
    }
}

// ---------------------------------------------------------------------------
// Path parsing
// ---------------------------------------------------------------------------

/// Split a `/api/environment/nodes/{id}/...` path into `(node_id, rest)`,
/// where `rest` is everything after the id segment (including the leading `/`).
pub(super) fn split_node_path<'a>(
    path: &'a str,
    prefix: &str,
) -> Result<(&'a str, &'a str), ConfigError> {
    let rest = path
        .strip_prefix(prefix)
        .ok_or_else(|| ConfigError::BadRequest("invalid environment node path".to_string()))?;
    let node_id = rest.split('/').next().unwrap_or("");
    if node_id.is_empty() {
        return Err(ConfigError::BadRequest(
            "invalid environment node path".to_string(),
        ));
    }
    let suffix = &rest[node_id.len()..];
    Ok((node_id, suffix))
}

/// Generate the next numeric node id by scanning every node in the tree.
pub(super) fn next_node_id(config: &TerminalServerConfig) -> String {
    config.environment.root.next_node_id()
}

// ---------------------------------------------------------------------------
// Request handlers (pure transforms; persistence is done by the caller)
// ---------------------------------------------------------------------------

/// Build a new node for insertion under `parent_id`. When `parent_id` is `None`
/// the node is intended to be attached to the root.
pub(super) fn create_node(
    config: &TerminalServerConfig,
    payload: WebCreateNodeRequest,
) -> Result<(Option<String>, EnvironmentNode), ConfigError> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(ConfigError::BadRequest(
            "environment node name cannot be empty".to_string(),
        ));
    }
    if let Some(parent_id) = &payload.parent_id {
        if parent_id.trim().is_empty() {
            return Err(ConfigError::BadRequest(
                "environment node parentId cannot be empty when provided".to_string(),
            ));
        }
        match config.environment.root.find_node(parent_id) {
            Some(parent) if parent.is_group() => {}
            Some(_) => {
                return Err(ConfigError::BadRequest(format!(
                    "environment node parent '{parent_id}' is not a group"
                )));
            }
            None => {
                return Err(ConfigError::BadRequest(format!(
                    "environment node parent '{parent_id}' not found"
                )));
            }
        }
    }

    let id = next_node_id(config);
    let node = match payload.kind {
        WebNodeKind::Group => EnvironmentNode::Group {
            id,
            name: name.to_string(),
            children: Vec::new(),
        },
        WebNodeKind::Leaf => EnvironmentNode::Leaf {
            id,
            name: name.to_string(),
            variables: Vec::new(),
        },
    };
    Ok((payload.parent_id, node))
}

/// Validate a move request and return `(target_parent_id, index)` on success.
/// Checks performed:
/// - the moved node exists;
/// - the target parent exists and is a group (or is `None` for root);
/// - the target is not inside the moved node's own subtree (would create a cycle);
/// - the target is not the moved node itself.
pub(super) fn validate_move(
    config: &TerminalServerConfig,
    node_id: &str,
    payload: &WebMoveNodeRequest,
) -> Result<(Option<String>, Option<usize>), ConfigError> {
    let node = config.environment.root.find_node(node_id).ok_or_else(|| {
        ConfigError::BadRequest(format!("environment node '{node_id}' not found"))
    })?;

    // Reject moving a node into itself.
    if let Some(target) = &payload.target_parent_id {
        if target == node_id {
            return Err(ConfigError::BadRequest(
                "cannot move an environment node into itself".to_string(),
            ));
        }
        // Reject moving a node into its own subtree (would create a cycle).
        let subtree_ids = node.subtree_ids();
        if subtree_ids.iter().any(|id| id == target) {
            return Err(ConfigError::BadRequest(
                "cannot move an environment node into its own subtree".to_string(),
            ));
        }
        match config.environment.root.find_node(target) {
            Some(parent) if parent.is_group() => {}
            Some(_) => {
                return Err(ConfigError::BadRequest(format!(
                    "environment move target '{target}' is not a group"
                )));
            }
            None => {
                return Err(ConfigError::BadRequest(format!(
                    "environment move target '{target}' not found"
                )));
            }
        }
    }

    Ok((payload.target_parent_id.clone(), payload.index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{EnvironmentConfig, EnvironmentVariable};

    fn group(id: &str, name: &str, children: Vec<EnvironmentNode>) -> EnvironmentNode {
        EnvironmentNode::Group {
            id: id.to_string(),
            name: name.to_string(),
            children,
        }
    }

    fn leaf(id: &str, name: &str) -> EnvironmentNode {
        EnvironmentNode::Leaf {
            id: id.to_string(),
            name: name.to_string(),
            variables: Vec::new(),
        }
    }

    fn root_with(children: Vec<EnvironmentNode>) -> EnvironmentGroup {
        EnvironmentGroup {
            id: "root".to_string(),
            name: "Root".to_string(),
            children,
        }
    }

    fn config_with(root: EnvironmentGroup) -> TerminalServerConfig {
        TerminalServerConfig {
            environment: EnvironmentConfig { root },
            ..Default::default()
        }
    }

    #[test]
    fn find_node_locates_nested_group_and_leaf() {
        let g1 = group(
            "1",
            "g1",
            vec![leaf("2", "l2"), group("3", "g3", vec![leaf("4", "l4")])],
        );
        let tree = root_with(vec![g1, leaf("5", "l5")]);

        assert!(tree.find_node("1").is_some());
        assert!(tree.find_node("2").is_some());
        assert!(tree.find_node("4").is_some());
        assert!(tree.find_node("5").is_some());
        assert!(tree.find_node("missing").is_none());
        // Root itself is never returned.
        assert!(tree.find_node("root").is_none());
    }

    #[test]
    fn collect_leaves_only_returns_leaf_nodes() {
        // root -> [g1 -> [l2, l3], l4, g5(leaf-less group)]
        let g1 = group("1", "g1", vec![leaf("2", "l2"), leaf("3", "l3")]);
        let g5 = group("5", "g5", vec![]);
        let tree = root_with(vec![g1, leaf("4", "l4"), g5]);

        let leaves: Vec<String> = tree
            .collect_leaves()
            .into_iter()
            .map(|n| n.id().to_string())
            .collect();
        // Empty groups are NOT leaves — only Leaf nodes are.
        assert_eq!(
            leaves,
            vec!["2".to_string(), "3".to_string(), "4".to_string()]
        );
    }

    #[test]
    fn remove_node_works_at_any_depth() {
        let g1 = group("1", "g1", vec![leaf("2", "l2"), leaf("3", "l3")]);
        let mut tree = root_with(vec![g1, leaf("4", "l4")]);

        assert!(tree.remove_node("3"));
        assert!(tree.find_node("3").is_none());
        assert!(tree.remove_node("4"));
        assert!(tree.find_node("4").is_none());
        assert!(!tree.remove_node("99"));
        assert!(!tree.remove_node("root"));
    }

    #[test]
    fn next_node_id_scans_whole_tree() {
        let g1 = group("2", "g1", vec![leaf("5", "l5")]);
        let tree = root_with(vec![g1, leaf("1", "l4")]);
        let config = config_with(tree);
        assert_eq!(next_node_id(&config), "6");
    }

    #[test]
    fn split_node_path_extracts_id_and_suffix() {
        assert_eq!(
            split_node_path(
                "/api/environment/nodes/42/variables",
                "/api/environment/nodes/"
            )
            .unwrap(),
            ("42", "/variables")
        );
        assert_eq!(
            split_node_path("/api/environment/nodes/7", "/api/environment/nodes/").unwrap(),
            ("7", "")
        );
        assert!(split_node_path("/api/environment/nodes/", "/api/environment/nodes/").is_err());
    }

    #[test]
    fn create_node_rejects_leaf_parent() {
        // Parent points at a Leaf, which cannot host children.
        let tree = root_with(vec![leaf("1", "l1")]);
        let config = config_with(tree);
        let payload = WebCreateNodeRequest {
            parent_id: Some("1".to_string()),
            kind: WebNodeKind::Group,
            name: "x".to_string(),
        };
        assert!(create_node(&config, payload).is_err());
    }

    #[test]
    fn create_node_accepts_root_parent_and_trims_name() {
        let config = TerminalServerConfig::default();
        let payload = WebCreateNodeRequest {
            parent_id: None,
            kind: WebNodeKind::Leaf,
            name: "  top  ".to_string(),
        };
        let (parent, node) = create_node(&config, payload).unwrap();
        assert!(parent.is_none());
        assert_eq!(node.name(), "top");
        assert_eq!(node.id(), "1");
        assert!(node.is_leaf());
    }

    #[test]
    fn validate_move_rejects_cycle_into_own_subtree() {
        // g1 contains l2; moving g1 into l2 is invalid (leaf target + cycle).
        let g1 = group("1", "g1", vec![leaf("2", "l2")]);
        let config = config_with(root_with(vec![g1]));
        let payload = WebMoveNodeRequest {
            target_parent_id: Some("2".to_string()),
            index: None,
        };
        let err = validate_move(&config, "1", &payload).unwrap_err();
        assert!(err.to_string().contains("subtree") || err.to_string().contains("not a group"));
    }

    #[test]
    fn validate_move_rejects_into_itself() {
        let g1 = group("1", "g1", vec![]);
        let config = config_with(root_with(vec![g1]));
        let payload = WebMoveNodeRequest {
            target_parent_id: Some("1".to_string()),
            index: None,
        };
        assert!(validate_move(&config, "1", &payload).is_err());
    }

    #[test]
    fn validate_move_accepts_root_target() {
        let g1 = group("1", "g1", vec![leaf("2", "l2")]);
        let config = config_with(root_with(vec![g1]));
        let payload = WebMoveNodeRequest {
            target_parent_id: None,
            index: Some(0),
        };
        let (target, index) = validate_move(&config, "2", &payload).unwrap();
        assert!(target.is_none());
        assert_eq!(index, Some(0));
    }

    #[test]
    fn legacy_entries_are_migrated_into_leaf_nodes() {
        let json = r#"{
            "entries": [
                { "id": "1", "name": "Prod", "variables": [ { "key": "K", "value": "" } ] }
            ]
        }"#;
        let config: EnvironmentConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.root.children.len(), 1);
        match &config.root.children[0] {
            EnvironmentNode::Leaf {
                id,
                name,
                variables,
            } => {
                assert_eq!(id, "1");
                assert_eq!(name, "Prod");
                assert_eq!(variables.len(), 1);
                assert_eq!(variables[0].key, "K");
            }
            other => panic!("expected Leaf, got {other:?}"),
        }
    }

    #[test]
    fn node_tagged_serialization_round_trips() {
        let _ = EnvironmentVariable {
            key: "x".into(),
            value: "y".into(),
        }; // keep import used
        let json = r#"{
            "root": {
                "id": "root",
                "name": "Root",
                "children": [
                    { "kind": "group", "id": "1", "name": "g1", "children": [
                        { "kind": "leaf", "id": "2", "name": "l2", "variables": [] }
                    ]},
                    { "kind": "leaf", "id": "3", "name": "l3", "variables": [] }
                ]
            }
        }"#;
        let config: EnvironmentConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.root.children.len(), 2);
        assert!(config.root.children[0].is_group());
        assert!(config.root.children[1].is_leaf());
    }

    fn mcp_config(enabled: bool, token: &str) -> crate::config::McpConfig {
        crate::config::McpConfig {
            enabled,
            access_token: token.to_string(),
            ..crate::config::McpConfig::default()
        }
    }

    /// Enabling MCP creates the managed leaf at the front of the root with
    /// the access token; a second sync with the same token is a no-op, a new
    /// token refreshes the value, and disabling removes the leaf again.
    #[test]
    fn sync_mcp_token_leaf_lifecycle() {
        let mut config = TerminalServerConfig::default();
        config.mcp = mcp_config(false, "secret");
        config.environment = EnvironmentConfig {
            root: root_with(vec![leaf("1", "user")]),
        };

        // Disabled: nothing happens even if the leaf is absent.
        assert!(!sync_mcp_token_leaf(&mut config));

        config.mcp = mcp_config(true, "secret");
        assert!(sync_mcp_token_leaf(&mut config));
        assert_eq!(config.environment.root.children.len(), 2);
        assert_eq!(config.environment.root.children[0].id(), MCP_TOKEN_LEAF_ID);
        assert_eq!(
            config.environment.root.children[0].name(),
            MCP_TOKEN_LEAF_NAME
        );
        let vars = config.environment.root.children[0]
            .as_leaf_variables()
            .unwrap();
        assert_eq!(
            vars,
            &[EnvironmentVariable {
                key: MCP_TOKEN_VARIABLE_KEY.to_string(),
                value: "secret".to_string(),
            }][..]
        );

        // Idempotent while the token is unchanged.
        assert!(!sync_mcp_token_leaf(&mut config));

        // A token change flows into the leaf.
        config.mcp = mcp_config(true, "rotated");
        assert!(sync_mcp_token_leaf(&mut config));
        let vars = config.environment.root.children[0]
            .as_leaf_variables()
            .unwrap();
        assert_eq!(vars[0].value, "rotated");

        // Disabling removes the leaf but keeps user nodes.
        config.mcp = mcp_config(false, "rotated");
        assert!(sync_mcp_token_leaf(&mut config));
        assert_eq!(config.environment.root.children.len(), 1);
        assert_eq!(config.environment.root.children[0].id(), "1");
        assert!(!sync_mcp_token_leaf(&mut config));
    }

    /// The managed leaf never disturbs numeric id allocation for user nodes.
    #[test]
    fn sync_mcp_token_leaf_keeps_numeric_ids_for_users() {
        let mut config = TerminalServerConfig::default();
        config.mcp = mcp_config(true, "secret");
        assert!(sync_mcp_token_leaf(&mut config));
        assert_eq!(next_node_id(&config), "1");
    }

    /// The web DTO marks only the managed leaf as readonly.
    #[test]
    fn web_node_marks_mcp_leaf_readonly() {
        let mut config = TerminalServerConfig::default();
        config.mcp = mcp_config(true, "secret");
        sync_mcp_token_leaf(&mut config);
        let web = web_environment_node(&config.environment.root.children[0]);
        assert!(matches!(
            &web,
            WebEnvironmentNode::Leaf { readonly: true, .. }
        ));
        let user = web_environment_node(&leaf("1", "user"));
        assert!(matches!(&user, WebEnvironmentNode::Leaf { .. }));
        if let WebEnvironmentNode::Leaf { readonly, .. } = user {
            assert!(!readonly);
        }
    }
}
