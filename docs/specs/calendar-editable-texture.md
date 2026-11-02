# Calendar editable event texture

Retail `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua:466-473` declares `EventSetTextureID(textureIndex)`. The original Calendar texture picker sets type and texture together.

WM39 describes local mutation: the required numeric index is reduced by one before truncation. Stored type 0 uses supplied list A; types 1 and 5 use list B. Other types and out-of-range indexes leave the record unchanged. An allowed selection copies the list row's ID, not its icon or metadata, and marks the record dirty only on a change. Announcement authority remains supplied state. Overflowing signed conversion uses the inferred masked-exception result; live exception behavior is unknown.

Wrapper completion, general return arity and indirect effects remain unknown. The original picker test uses distinct list IDs and icon IDs.
