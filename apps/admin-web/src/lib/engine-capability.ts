import {
  decodeEngineCapabilityList,
  isEngineCapabilityList,
  type ContractEngineCapability,
  type ContractEngineCapabilityList
} from "@selemene/engine-sdk";

export const PUBLIC_MIRROR_IDS = [
  "biofield",
  "biorhythm",
  "enneagram",
  "face-reading",
  "gene-keys",
  "human-design",
  "i-ching",
  "nadabrahman",
  "numerology",
  "panchanga",
  "raaga",
  "sacred-geometry",
  "sigil-forge",
  "tarot",
  "transits",
  "vedic-clock",
  "vimshottari"
] as const;

export type AdminCapability = ContractEngineCapability & {
  public_mirror: boolean;
};

export type AdminCapabilityList = Omit<ContractEngineCapabilityList, "capabilities"> & {
  capabilities: AdminCapability[];
};

export function decodeAdminCapabilityList(payload: unknown): AdminCapabilityList {
  const decoded = decodeEngineCapabilityList(payload);
  const capabilities = decoded.capabilities.map((capability) => ({
    ...capability,
    public_mirror: (PUBLIC_MIRROR_IDS as readonly string[]).includes(capability.engine_id)
  }));
  if (capabilities.filter((capability) => capability.public_mirror).length !== 17) {
    throw new TypeError("canonical capability list public mirror projection is malformed");
  }
  return { ...decoded, capabilities };
}

export function isAdminCapabilityList(payload: unknown): payload is AdminCapabilityList {
  if (!isEngineCapabilityList(payload)) return false;
  try {
    decodeAdminCapabilityList(payload);
    return true;
  } catch {
    return false;
  }
}

/** Preserve the old admin array shape during the consumer migration. */
export function toLegacyAdminCapabilities(list: AdminCapabilityList): Array<{
  engine_id: string;
  engine_name: string;
  required_phase: number;
  category: string;
  status: string;
}> {
  return list.capabilities.map((capability) => ({
    engine_id: capability.engine_id,
    engine_name: capability.display_name,
    required_phase: capability.required_phase ?? 0,
    category: capability.runtime_kind,
    status: capability.availability
  }));
}

export function capabilityStateLabel(capability: AdminCapability): string {
  if (capability.engine_id === "biofield-capture" && capability.availability === "declared") {
    return "declared (database-conditional)";
  }
  return capability.availability;
}
