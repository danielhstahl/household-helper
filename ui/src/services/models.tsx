export type QueryLatency = {
  index: number;
  range: string;
  count: number;
};
export type QueryTools = {
  cnt_spns_with_tools: number;
  cnt_spns_without_tools: number;
  date: Date;
};
export type TelemetryMetrics = {
  queryLatency: readonly QueryLatency[];
  ingestionLatency: readonly QueryLatency[];
  queryTools: readonly QueryTools[];
};

export interface Token {
  access_token: string;
}

export const RoleTypeEnum = {
  admin: "Admin",
  helper: "Helper",
  tutor: "Tutor",
} as const;

export type RoleType = (typeof RoleTypeEnum)[keyof typeof RoleTypeEnum];

export interface UserResponse {
  id: string;
  username: string;
  roles: RoleType[];
}

export const MessageTypeEnum = {
  human: "HumanMessage",
  ai: "AIMessage",
  system: "SystemMessage",
  tool: "ToolMessage",
} as const;

export type MessageType =
  (typeof MessageTypeEnum)[keyof typeof MessageTypeEnum];

export interface Message {
  message_type: MessageType;
  content: string;
  timestamp: string;
}

export interface Session {
  id: string;
  session_start: string;
}

export interface SessionDB extends Session {
  username_id: string;
}

export interface KnowledgeBase {
  id: number;
  name: string;
}
