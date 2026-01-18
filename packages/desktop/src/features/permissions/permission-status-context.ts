// features/permissions/permission-status-context.ts - Permission status context (separate for Fast Refresh)

import { createContext } from "react";
import type { PermissionContextValue } from "./types";

export const PermissionStatusContext = createContext<PermissionContextValue | undefined>(undefined);
