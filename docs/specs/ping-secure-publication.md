# Retail secure ping publication

Retail 12.1.5 documents `C_PingSecure` as `SecureOnly` in `Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:3–6`. The pinned `Blizzard_PingUI/Blizzard_PingUI.toc` loads PingUI into the secure environment. Build 70688 WM38 registration evidence supports a separate secure namespace, not an ordinary public-global installation.

The simulator publishes the existing `C_PingSecure` namespace in the secure environment and removes its public-global binding for Retail 12.1.5. Earlier Retail epochs and other profiles are unchanged. This publication contract does not by itself claim that every member's native behavior is modeled. Other client code could install an alias; such aliases are not modeled here.
