#!/bin/bash

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <Info.plist> [--check]" >&2
  exit 64
fi

PLIST_PATH="$1"
MODE="${2:-}"
PLIST_BUDDY="/usr/libexec/PlistBuddy"

if [[ ! -f "$PLIST_PATH" ]]; then
  echo "Info.plist not found: $PLIST_PATH" >&2
  exit 66
fi

contains_json_document_type() {
  local output

  if ! output="$("$PLIST_BUDDY" -c "Print :CFBundleDocumentTypes" "$PLIST_PATH" 2>/dev/null)"; then
    return 1
  fi

  [[ "$output" == *"public.json"* ]] &&
  [[ "$output" == *"JSON Document"* ]] &&
  [[ "$output" == *"Viewer"* ]] &&
  [[ "$output" == *"Alternate"* ]]
}

if [[ "$MODE" == "--check" ]]; then
  contains_json_document_type
  exit $?
fi

"$PLIST_BUDDY" -c "Delete :CFBundleDocumentTypes" "$PLIST_PATH" >/dev/null 2>&1 || true
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes array" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0 dict" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0:CFBundleTypeName string JSON Document" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0:CFBundleTypeRole string Viewer" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0:LSHandlerRank string Alternate" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0:LSItemContentTypes array" "$PLIST_PATH"
"$PLIST_BUDDY" -c "Add :CFBundleDocumentTypes:0:LSItemContentTypes:0 string public.json" "$PLIST_PATH"

plutil -lint "$PLIST_PATH" >/dev/null
