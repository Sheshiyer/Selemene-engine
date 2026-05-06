"use client";

import { useEffect, useState } from "react";
import {
  QuotaExceededModal,
  type QuotaExceededDetail,
} from "./QuotaExceededModal";
import { subscribeToQuotaExceeded } from "@/lib/quota";

/**
 * Mounts the QuotaExceededModal globally. Subscribes to quota events fired
 * from src/lib/api.ts when an API call returns 402 with error_code
 * QUOTA_EXCEEDED. Stays mounted as a singleton in the root layout.
 */
export function QuotaExceededHost() {
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<QuotaExceededDetail | undefined>();

  useEffect(() => {
    return subscribeToQuotaExceeded((d) => {
      setDetail(d);
      setOpen(true);
    });
  }, []);

  return (
    <QuotaExceededModal
      open={open}
      detail={detail}
      onDismiss={() => setOpen(false)}
    />
  );
}
