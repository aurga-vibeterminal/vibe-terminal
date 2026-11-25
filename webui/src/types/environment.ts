export interface EnvironmentVariable {
  key: string;
  value: string;
}

/**
 * A node in the environment tree. The `kind` discriminator is serialised as
 * `"group"` / `"leaf"`. Groups are pure folders (never hold variables); Leaves
 * are environment-variable collections that can be applied to a terminal
 * session. The two are strictly separated.
 */
export interface EnvironmentGroup {
  kind: 'group';
  id: string;
  name: string;
  children: EnvironmentNode[];
}

export interface EnvironmentLeaf {
  kind: 'leaf';
  id: string;
  name: string;
  variables: EnvironmentVariable[];
  /** Server-managed leaf (e.g. the MCP bearer token); displayed read-only. */
  readonly?: boolean;
}

export type EnvironmentNode = EnvironmentGroup | EnvironmentLeaf;

export interface EnvironmentConfig {
  root: EnvironmentGroup;
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

export type EnvironmentNodeKind = 'group' | 'leaf';

/** Body for `POST /api/environment/nodes`. */
export interface CreateNodeRequest {
  /** Parent group id; `null` attaches the new node to the synthetic root. */
  parentId: string | null;
  kind: EnvironmentNodeKind;
  name: string;
}

/** Body for `PUT /api/environment/nodes/{id}`. */
export interface RenameNodeRequest {
  name: string;
}

/** Body for `PUT /api/environment/nodes/{id}/variables`. */
export interface UpdateLeafRequest {
  variables: EnvironmentVariable[];
}

/** Body for `POST /api/environment/nodes/{id}/move`. */
export interface MoveNodeRequest {
  /** Destination parent group id; `null` moves the node to the root level. */
  targetParentId: string | null;
  /** 0-based insertion index among the destination's children; `null` appends. */
  index: number | null;
}
