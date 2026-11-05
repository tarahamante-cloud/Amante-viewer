# Local media and Blender security

This document covers the local-peer remediation only. The older broad audit
was removed during the clean upstream reset; its custom updater, TLS and
credential conclusions must not be read as guarantees about the current build.

- Media TCP connections must belong to the exact launched SLPlugin child.
  Linux verifies the endpoint/socket inode against the child's procfs file
  descriptors; Windows verifies both endpoints using the TCP owner-PID table.
  Lookup failures and unsupported platforms deny access without a fallback.
- Blender requires mutual per-session pairing before rig disclosure, scene
  changes or shape/import controls. The viewer and addon must be upgraded
  together; no unauthenticated legacy mode is provided.
- Media XML frames, pending and received queues, per-idle socket/parse work,
  partial-frame deadlines, texture geometry and shared memory are bounded.
- Shared-memory names and Windows ACLs remain a separate boundary requiring
  desktop validation. Socket authorization does not isolate native trusted
  plugins or prevent an administrator/debugger from reading process memory.

See [LOCAL_IPC.md](LOCAL_IPC.md) for limits, protocol compatibility, synthetic
peer tests, native verification evidence and remaining platform/runtime limits.
Real plugin/Blender/GPU testing is separate from source and compile checks.
This work does not activate media downloads or alter updater trust policy.
