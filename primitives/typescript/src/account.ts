export type ApiPlan = "Free" | "Starter" | "Growth" | "Scale" | "Enterprise";
export type PaidPlanDuration = "Month" | "Quarter" | "HalfYear" | "Year";

export interface ApiAccount {
  metadata: {
    plan: ApiPlan;
    duration: PaidPlanDuration;
    credits: number;
    img_credits: number;
    expiry_at: number;
  };
  keys: { name: string; created_at: number }[];
}

export interface UserProfile {
  name?: string;
  nickname?: string;
  email?: string;
  preferred_fiat_currency?: "USD" | "INR" | "EUR";
}

export type Platform =
  | "TgBot" | "Web" | "CliMac" | "CliLinux" | "CliWindows"
  | "MacDesktop" | "LinuxDesktop" | "WindowsDesktop";

export interface WalletAuthChallengeRequest {
  domain: string;
  uri: string;
  chain_id: number;
}

export interface WalletAuthChallenge {
  message: string;
  nonce: string;
  expires_at: number;
}

export interface WalletAuthProof {
  message: string;
  signature: string;
  nonce: string;
  platform: Platform;
}

export interface FalconHeartbeat {
  ipo?: { enabled: boolean; freq_secs: number };
  spo_enabled: boolean;
}

export interface UserSession {
  user_id: string;
  jwt: string;
  falcon_heartbeat?: FalconHeartbeat;
}
