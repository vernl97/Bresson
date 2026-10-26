local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrBinding"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrDialogs"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrFileUtils"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrPathUtils"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrView"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrLogger"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrApplication"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrTasks"
L8_1 = L8_1(L9_1)
L9_1 = require
L10_1 = "UtilV3"
L9_1 = L9_1(L10_1)
L10_1 = require
L11_1 = "ExifV3"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrColor"
L11_1 = L11_1(L12_1)
L12_1 = L6_1
L13_1 = "NegativeExport"
L12_1 = L12_1(L13_1)
L13_1 = _G
L13_1 = L13_1.dev
if L13_1 == true then
  L14_1 = L12_1
  L13_1 = L12_1.enable
  L15_1 = "logfile"
  L13_1(L14_1, L15_1)
else
  L14_1 = L12_1
  L13_1 = L12_1.disable
  L13_1(L14_1)
end
L13_1 = L5_1.bind
L14_1 = require
L15_1 = "MetadataDefinitionsV3"
L14_1 = L14_1(L15_1)
L15_1 = L14_1.metadataFieldsForPhotos
L16_1 = L7_1.versionTable
L16_1 = L16_1()
L17_1 = {}
L18_1 = {}
L19_1 = "\""
L20_1 = MAC_ENV
if L20_1 then
  L20_1 = "/mac/exiftool"
  if L20_1 then
    goto lbl_65
  end
end
L20_1 = "win\\exiftool.exe"
::lbl_65::
L21_1 = string
L21_1 = L21_1.format
L22_1 = "%s%s%s%s"
L23_1 = L19_1
L24_1 = _PLUGIN
L24_1 = L24_1.path
L25_1 = "/mac/exiftool"
L26_1 = L19_1
L21_1 = L21_1(L22_1, L23_1, L24_1, L25_1, L26_1)
L22_1 = WIN_ENV
if L22_1 then
  L22_1 = L9_1.cdCommand
  L23_1 = _PLUGIN
  L23_1 = L23_1.path
  L22_1 = L22_1(L23_1)
  L23_1 = " "
  L24_1 = L20_1
  L21_1 = L22_1 .. L23_1 .. L24_1
end
L22_1 = L7_1.activeCatalog
L22_1 = L22_1()
L23_1 = {}
L24_1 = {}
L24_1.key = "writeToExif"
L24_1.default = true
L25_1 = {}
L25_1.key = "generateKeywords"
L25_1.default = true
L26_1 = {}
L26_1.key = "reimportWithMetadata"
L26_1.default = true
L27_1 = {}
L27_1.key = "includeEquipmentCaption"
L27_1.default = true
L28_1 = {}
L28_1.key = "includeScanningCaption"
L28_1.default = true
L29_1 = {}
L29_1.key = "includeDevelopmentCaption"
L29_1.default = true
L30_1 = {}
L30_1.key = "includeNLPCaption"
L30_1.default = true
L31_1 = {}
L31_1.key = "appendCaption"
L31_1.default = true
L32_1 = {}
L32_1.key = "copyCaption"
L32_1.default = true
L33_1 = {}
L33_1.key = "captionSeparator"
L33_1.default = " | "
L23_1[1] = L24_1
L23_1[2] = L25_1
L23_1[3] = L26_1
L23_1[4] = L27_1
L23_1[5] = L28_1
L23_1[6] = L29_1
L23_1[7] = L30_1
L23_1[8] = L31_1
L23_1[9] = L32_1
L23_1[10] = L33_1
L18_1.exportPresetFields = L23_1
L18_1.canExportVideo = false
L23_1 = {}
L24_1 = "video"
L23_1[1] = L24_1
L18_1.hideSections = L23_1
L23_1 = {}
L24_1 = "JPEG"
L25_1 = "TIFF"
L23_1[1] = L24_1
L23_1[2] = L25_1
L18_1.allowFileFormats = L23_1

function L23_1(A0_2)
  local L1_2
end

L18_1.startDialog = L23_1

function L23_1(A0_2, A1_2)
end

L18_1.endDialog = L23_1

function L23_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2
  L2_2 = {}
  L3_2 = {}
  L3_2.title = "Negative Lab Pro | Export Overview"
  L3_2.synopsis = ""
  L5_2 = A0_2
  L4_2 = A0_2.column
  L6_2 = {}
  L8_2 = A0_2
  L7_2 = A0_2.control_spacing
  L7_2 = L7_2(L8_2)
  L7_2 = L7_2 / 2
  L6_2.spacing = L7_2
  L8_2 = A0_2
  L7_2 = A0_2.static_text
  L9_2 = {}
  L9_2.title = "There's so much rich, beautiful data surrounding your film captures, development and digitization. Let's be sure to capture it!"
  L9_2.height_in_lines = 2
  L9_2.width_in_chars = 50
  L9_2.wraps = true
  L10_2 = L11_1
  L11_2 = 0.4
  L12_2 = 0.4
  L13_2 = 0.4
  L10_2 = L10_2(L11_2, L12_2, L13_2)
  L9_2.text_color = L10_2
  L7_2 = L7_2(L8_2, L9_2)
  L9_2 = A0_2
  L8_2 = A0_2.static_text
  L10_2 = {}
  L10_2.title = "So before you begin your export, fill in any relevant data in the custom 'Negative Lab Pro' metadata section."
  L10_2.height_in_lines = 2
  L10_2.width_in_chars = 50
  L10_2.wraps = true
  L11_2 = L11_1
  L12_2 = 0.4
  L13_2 = 0.4
  L14_2 = 0.4
  L11_2 = L11_2(L12_2, L13_2, L14_2)
  L10_2.text_color = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.picture
  L11_2 = {}
  L12_2 = L4_1
  L12_2 = L12_2.child
  L13_2 = _PLUGIN
  L13_2 = L13_2.path
  L14_2 = "img/nlp-meta.png"
  L12_2 = L12_2(L13_2, L14_2)
  L11_2.value = L12_2
  L11_2.width = 260
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.static_text
  L12_2 = {}
  L12_2.title = "You can access this in the 'Library' module. On the right hand pane, click the dropdown beside 'Metadata' and select 'Negative Lab Pro.' These info you enter will then be populated into standard metadata during export, using the options below."
  L12_2.height_in_lines = 3
  L12_2.width_in_chars = 50
  L12_2.wraps = true
  L13_2 = L11_1
  L14_2 = 0.4
  L15_2 = 0.4
  L16_2 = 0.4
  L13_2 = L13_2(L14_2, L15_2, L16_2)
  L12_2.text_color = L13_2
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L10_2(L11_2, L12_2)
  L6_2[1] = L7_2
  L6_2[2] = L8_2
  L6_2[3] = L9_2
  L6_2[4] = L10_2
  L6_2[5] = L11_2
  L6_2[6] = L12_2
  L6_2[7] = L13_2
  L6_2[8] = L14_2
  L6_2[9] = L15_2
  L6_2[10] = L16_2
  L6_2[11] = L17_2
  L6_2[12] = L18_2
  L6_2[13] = L19_2
  L6_2[14] = L20_2
  L6_2[15] = L21_2
  L6_2[16] = L22_2
  L6_2[17] = L23_2
  L6_2[18] = L24_2
  L6_2[19] = L25_2
  L6_2[20] = L26_2
  L6_2[21] = L27_2
  L6_2[22] = L28_2
  L6_2[23] = L29_2
  L6_2[24] = L30_2
  L6_2[25] = L31_2
  L6_2[26] = L32_2
  L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L4_2(L5_2, L6_2)
  L3_2[1] = L4_2
  L3_2[2] = L5_2
  L3_2[3] = L6_2
  L3_2[4] = L7_2
  L3_2[5] = L8_2
  L3_2[6] = L9_2
  L3_2[7] = L10_2
  L3_2[8] = L11_2
  L3_2[9] = L12_2
  L3_2[10] = L13_2
  L3_2[11] = L14_2
  L3_2[12] = L15_2
  L3_2[13] = L16_2
  L3_2[14] = L17_2
  L3_2[15] = L18_2
  L3_2[16] = L19_2
  L3_2[17] = L20_2
  L3_2[18] = L21_2
  L3_2[19] = L22_2
  L3_2[20] = L23_2
  L3_2[21] = L24_2
  L3_2[22] = L25_2
  L3_2[23] = L26_2
  L3_2[24] = L27_2
  L3_2[25] = L28_2
  L3_2[26] = L29_2
  L3_2[27] = L30_2
  L3_2[28] = L31_2
  L3_2[29] = L32_2
  L4_2 = {}
  L4_2.title = "Negative Lab Pro | Metadata"
  L4_2.synopsis = ""
  L6_2 = A0_2
  L5_2 = A0_2.column
  L7_2 = {}
  L9_2 = A0_2
  L8_2 = A0_2.control_spacing
  L8_2 = L8_2(L9_2)
  L8_2 = L8_2 / 2
  L7_2.spacing = L8_2
  L7_2.fill_horizontal = 1
  L9_2 = A0_2
  L8_2 = A0_2.checkbox
  L10_2 = {}
  L10_2.enabled = true
  L10_2.title = "Write EXIF Fields with Custom Metadata"
  L10_2.alignment = "left"
  L11_2 = L13_1
  L12_2 = "writeToExif"
  L11_2 = L11_2(L12_2)
  L10_2.value = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.static_text
  L11_2 = {}
  L11_2.title = "Standard EXIF fields (camera make, model, lens, focal length, film iso, date, exposure) will be populated with the custom film metadata you've added."
  L11_2.height_in_lines = 3
  L11_2.width_in_chars = 50
  L11_2.wraps = true
  L12_2 = L11_1
  L13_2 = 0.4
  L14_2 = 0.4
  L15_2 = 0.4
  L12_2 = L12_2(L13_2, L14_2, L15_2)
  L11_2.text_color = L12_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.checkbox
  L12_2 = {}
  L12_2.enabled = true
  L12_2.title = "Automatically Generate Keywords"
  L12_2.alignment = "left"
  L13_2 = L13_1
  L14_2 = "generateKeywords"
  L13_2 = L13_2(L14_2)
  L12_2.value = L13_2
  L10_2 = L10_2(L11_2, L12_2)
  L12_2 = A0_2
  L11_2 = A0_2.static_text
  L13_2 = {}
  L13_2.title = "Automatically creates keywords based on the information you provide in the custom metadata section."
  L13_2.height_in_lines = 3
  L13_2.width_in_chars = 50
  L13_2.wraps = true
  L14_2 = L11_1
  L15_2 = 0.4
  L16_2 = 0.4
  L17_2 = 0.4
  L14_2 = L14_2(L15_2, L16_2, L17_2)
  L13_2.text_color = L14_2
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.checkbox
  L14_2 = {}
  L14_2.enabled = true
  L14_2.title = "Retain Custom LR Metadata"
  L14_2.alignment = "left"
  L15_2 = L13_1
  L16_2 = "reimportWithMetadata"
  L15_2 = L15_2(L16_2)
  L14_2.value = L15_2
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.static_text
  L15_2 = {}
  L15_2.title = "NOTE: This will only work when 'add to this catalog' is also selected in the 'Export Location' options."
  L15_2.height_in_lines = 3
  L15_2.width_in_chars = 50
  L15_2.wraps = true
  L16_2 = L11_1
  L17_2 = 0.4
  L18_2 = 0.4
  L19_2 = 0.4
  L16_2 = L16_2(L17_2, L18_2, L19_2)
  L15_2.text_color = L16_2
  L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L13_2(L14_2, L15_2)
  L7_2[1] = L8_2
  L7_2[2] = L9_2
  L7_2[3] = L10_2
  L7_2[4] = L11_2
  L7_2[5] = L12_2
  L7_2[6] = L13_2
  L7_2[7] = L14_2
  L7_2[8] = L15_2
  L7_2[9] = L16_2
  L7_2[10] = L17_2
  L7_2[11] = L18_2
  L7_2[12] = L19_2
  L7_2[13] = L20_2
  L7_2[14] = L21_2
  L7_2[15] = L22_2
  L7_2[16] = L23_2
  L7_2[17] = L24_2
  L7_2[18] = L25_2
  L7_2[19] = L26_2
  L7_2[20] = L27_2
  L7_2[21] = L28_2
  L7_2[22] = L29_2
  L7_2[23] = L30_2
  L7_2[24] = L31_2
  L7_2[25] = L32_2
  L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L5_2(L6_2, L7_2)
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L4_2[4] = L8_2
  L4_2[5] = L9_2
  L4_2[6] = L10_2
  L4_2[7] = L11_2
  L4_2[8] = L12_2
  L4_2[9] = L13_2
  L4_2[10] = L14_2
  L4_2[11] = L15_2
  L4_2[12] = L16_2
  L4_2[13] = L17_2
  L4_2[14] = L18_2
  L4_2[15] = L19_2
  L4_2[16] = L20_2
  L4_2[17] = L21_2
  L4_2[18] = L22_2
  L4_2[19] = L23_2
  L4_2[20] = L24_2
  L4_2[21] = L25_2
  L4_2[22] = L26_2
  L4_2[23] = L27_2
  L4_2[24] = L28_2
  L4_2[25] = L29_2
  L4_2[26] = L30_2
  L4_2[27] = L31_2
  L4_2[28] = L32_2
  L5_2 = {}
  L5_2.title = "Negative Lab Pro | Auto-Caption"
  L5_2.synopsis = ""
  L7_2 = A0_2
  L6_2 = A0_2.column
  L8_2 = {}
  L10_2 = A0_2
  L9_2 = A0_2.control_spacing
  L9_2 = L9_2(L10_2)
  L9_2 = L9_2 / 2
  L8_2.spacing = L9_2
  L8_2.fill_horizontal = 1
  L10_2 = A0_2
  L9_2 = A0_2.static_text
  L11_2 = {}
  L11_2.title = "Negative Lab Pro can automatically generate captions based on the detail you provide in the custom metadata. This is useful because standard EXIF fields are limited. Some photo services (like flickr, or 500px) will automatically use this caption."
  L11_2.height_in_lines = 4
  L11_2.width_in_chars = 50
  L11_2.wraps = true
  L12_2 = L11_1
  L13_2 = 0.4
  L14_2 = 0.4
  L15_2 = 0.4
  L12_2 = L12_2(L13_2, L14_2, L15_2)
  L11_2.text_color = L12_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.checkbox
  L12_2 = {}
  L12_2.enabled = true
  L12_2.title = "Include Analog Equipment Caption"
  L12_2.alignment = "left"
  L13_2 = L13_1
  L14_2 = "includeEquipmentCaption"
  L13_2 = L13_2(L14_2)
  L12_2.value = L13_2
  L10_2 = L10_2(L11_2, L12_2)
  L12_2 = A0_2
  L11_2 = A0_2.static_text
  L13_2 = {}
  L13_2.title = "ex. 'Hasselblad 500cm | 80mm 2.8 | Portra 400'"
  L13_2.height_in_lines = 2
  L13_2.width_in_chars = 50
  L13_2.wraps = true
  L14_2 = L11_1
  L15_2 = 0.4
  L16_2 = 0.4
  L17_2 = 0.4
  L14_2 = L14_2(L15_2, L16_2, L17_2)
  L13_2.text_color = L14_2
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.checkbox
  L14_2 = {}
  L14_2.enabled = true
  L14_2.title = "Include Scanning / Digitization Caption"
  L14_2.alignment = "left"
  L15_2 = L13_1
  L16_2 = "includeScanningCaption"
  L15_2 = L15_2(L16_2)
  L14_2.value = L15_2
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.static_text
  L15_2 = {}
  L15_2.title = "ex. 'Scanned with Epson V800 + Negative Lab Pro v2.1'"
  L15_2.height_in_lines = 2
  L15_2.width_in_chars = 50
  L15_2.wraps = true
  L16_2 = L11_1
  L17_2 = 0.4
  L18_2 = 0.4
  L19_2 = 0.4
  L16_2 = L16_2(L17_2, L18_2, L19_2)
  L15_2.text_color = L16_2
  L13_2 = L13_2(L14_2, L15_2)
  L15_2 = A0_2
  L14_2 = A0_2.checkbox
  L16_2 = {}
  L16_2.enabled = true
  L16_2.title = "Include Film Development Caption"
  L16_2.alignment = "left"
  L17_2 = L13_1
  L18_2 = "includeDevelopmentCaption"
  L17_2 = L17_2(L18_2)
  L16_2.value = L17_2
  L14_2 = L14_2(L15_2, L16_2)
  L16_2 = A0_2
  L15_2 = A0_2.static_text
  L17_2 = {}
  L17_2.title = "ex. 'Home developed in Kodak HC-110 1+30 | 16 minutes @ 20 degrees'"
  L17_2.height_in_lines = 2
  L17_2.width_in_chars = 50
  L17_2.wraps = true
  L18_2 = L11_1
  L19_2 = 0.4
  L20_2 = 0.4
  L21_2 = 0.4
  L18_2 = L18_2(L19_2, L20_2, L21_2)
  L17_2.text_color = L18_2
  L15_2 = L15_2(L16_2, L17_2)
  L17_2 = A0_2
  L16_2 = A0_2.checkbox
  L18_2 = {}
  L18_2.enabled = true
  L18_2.title = "Include Negative Lab Pro Caption"
  L18_2.alignment = "left"
  L19_2 = L13_1
  L20_2 = "includeNLPCaption"
  L19_2 = L19_2(L20_2)
  L18_2.value = L19_2
  L16_2 = L16_2(L17_2, L18_2)
  L18_2 = A0_2
  L17_2 = A0_2.static_text
  L19_2 = {}
  L19_2.title = "ex. 'Negative Lab Pro v2.2 | WB: Auto-Warm | Tone Profile: Cinematic | LUT: Pakon'"
  L19_2.height_in_lines = 2
  L19_2.width_in_chars = 50
  L19_2.wraps = true
  L20_2 = L11_1
  L21_2 = 0.4
  L22_2 = 0.4
  L23_2 = 0.4
  L20_2 = L20_2(L21_2, L22_2, L23_2)
  L19_2.text_color = L20_2
  L17_2 = L17_2(L18_2, L19_2)
  L19_2 = A0_2
  L18_2 = A0_2.checkbox
  L20_2 = {}
  L20_2.enabled = true
  L20_2.title = "Append To Existing Caption"
  L20_2.alignment = "left"
  L21_2 = L13_1
  L22_2 = "appendCaption"
  L21_2 = L21_2(L22_2)
  L20_2.value = L21_2
  L18_2 = L18_2(L19_2, L20_2)
  L20_2 = A0_2
  L19_2 = A0_2.static_text
  L21_2 = {}
  L21_2.title = "If you've already written a caption, this will append the auto-caption to the end. If left unchecked, it will overwrite any existing caption in the exported file."
  L21_2.height_in_lines = 3
  L21_2.width_in_chars = 50
  L21_2.wraps = true
  L22_2 = L11_1
  L23_2 = 0.4
  L24_2 = 0.4
  L25_2 = 0.4
  L22_2 = L22_2(L23_2, L24_2, L25_2)
  L21_2.text_color = L22_2
  L19_2 = L19_2(L20_2, L21_2)
  L21_2 = A0_2
  L20_2 = A0_2.checkbox
  L22_2 = {}
  L22_2.enabled = true
  L22_2.title = "Copy Caption to Clipboard"
  L22_2.alignment = "left"
  L23_2 = L13_1
  L24_2 = "copyCaption"
  L23_2 = L23_2(L24_2)
  L22_2.value = L23_2
  L20_2 = L20_2(L21_2, L22_2)
  L22_2 = A0_2
  L21_2 = A0_2.static_text
  L23_2 = {}
  L23_2.title = "The caption output of the first exported image will be copied to your clipboard. You can then paste into Facebook, Instagram, etc."
  L23_2.height_in_lines = 3
  L23_2.width_in_chars = 50
  L23_2.wraps = true
  L24_2 = L11_1
  L25_2 = 0.4
  L26_2 = 0.4
  L27_2 = 0.4
  L24_2 = L24_2(L25_2, L26_2, L27_2)
  L23_2.text_color = L24_2
  L21_2 = L21_2(L22_2, L23_2)
  L23_2 = A0_2
  L22_2 = A0_2.row
  L24_2 = {}
  L26_2 = A0_2
  L25_2 = A0_2.static_text
  L27_2 = {}
  L27_2.title = "Separator:"
  L25_2 = L25_2(L26_2, L27_2)
  L27_2 = A0_2
  L26_2 = A0_2.popup_menu
  L28_2 = {}
  L29_2 = L13_1
  L30_2 = "captionSeparator"
  L29_2 = L29_2(L30_2)
  L28_2.value = L29_2
  L29_2 = {}
  L30_2 = {}
  L30_2.title = "Comma ( , )"
  L30_2.value = ", "
  L31_2 = {}
  L31_2.title = "Vertical Line ( | )"
  L31_2.value = " | "
  L32_2 = {}
  L32_2.title = "Forward Slashes ( // )"
  L32_2.value = " // "
  L29_2[1] = L30_2
  L29_2[2] = L31_2
  L29_2[3] = L32_2
  L28_2.items = L29_2
  L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L26_2(L27_2, L28_2)
  L24_2[1] = L25_2
  L24_2[2] = L26_2
  L24_2[3] = L27_2
  L24_2[4] = L28_2
  L24_2[5] = L29_2
  L24_2[6] = L30_2
  L24_2[7] = L31_2
  L24_2[8] = L32_2
  L22_2 = L22_2(L23_2, L24_2)
  L24_2 = A0_2
  L23_2 = A0_2.static_text
  L25_2 = {}
  L25_2.title = "Used to separate items in each caption group. For instance, 'Hasselblad 500cm // 80mm 2.8' vs 'Hasselblad 500cm, 80mm 2.8'"
  L25_2.height_in_lines = 3
  L25_2.width_in_chars = 50
  L25_2.wraps = true
  L26_2 = L11_1
  L27_2 = 0.4
  L28_2 = 0.4
  L29_2 = 0.4
  L26_2 = L26_2(L27_2, L28_2, L29_2)
  L25_2.text_color = L26_2
  L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L23_2(L24_2, L25_2)
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L8_2[4] = L12_2
  L8_2[5] = L13_2
  L8_2[6] = L14_2
  L8_2[7] = L15_2
  L8_2[8] = L16_2
  L8_2[9] = L17_2
  L8_2[10] = L18_2
  L8_2[11] = L19_2
  L8_2[12] = L20_2
  L8_2[13] = L21_2
  L8_2[14] = L22_2
  L8_2[15] = L23_2
  L8_2[16] = L24_2
  L8_2[17] = L25_2
  L8_2[18] = L26_2
  L8_2[19] = L27_2
  L8_2[20] = L28_2
  L8_2[21] = L29_2
  L8_2[22] = L30_2
  L8_2[23] = L31_2
  L8_2[24] = L32_2
  L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2 = L6_2(L7_2, L8_2)
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L5_2[4] = L9_2
  L5_2[5] = L10_2
  L5_2[6] = L11_2
  L5_2[7] = L12_2
  L5_2[8] = L13_2
  L5_2[9] = L14_2
  L5_2[10] = L15_2
  L5_2[11] = L16_2
  L5_2[12] = L17_2
  L5_2[13] = L18_2
  L5_2[14] = L19_2
  L5_2[15] = L20_2
  L5_2[16] = L21_2
  L5_2[17] = L22_2
  L5_2[18] = L23_2
  L5_2[19] = L24_2
  L5_2[20] = L25_2
  L5_2[21] = L26_2
  L5_2[22] = L27_2
  L5_2[23] = L28_2
  L5_2[24] = L29_2
  L5_2[25] = L30_2
  L5_2[26] = L31_2
  L5_2[27] = L32_2
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  return L2_2
end

L18_1.sectionsForTopOfDialog = L23_1

function L23_1(A0_2)
  local L1_2, L2_2
  L1_2 = A0_2.reimportWithMetadata
  if L1_2 == true then
    L1_2 = L17_1
    L2_2 = A0_2.LR_reimportExportedPhoto
    L1_2.LR_reimportExportedPhoto = L2_2
    A0_2.LR_reimportExportedPhoto = false
  end
end

L18_1.updateExportSettings = L23_1

function L23_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2
  L2_2 = A1_2.exportSession
  L3_2 = A1_2.propertyTable
  L5_2 = L2_2
  L4_2 = L2_2.countRenditions
  L4_2 = L4_2(L5_2)
  L6_2 = A1_2
  L5_2 = A1_2.configureProgress
  L7_2 = {}
  if 1 < L4_2 then
    L8_2 = "Exporting "
    L9_2 = L4_2
    L10_2 = " photos"
    L8_2 = L8_2 .. L9_2 .. L10_2
    if L8_2 then
      goto lbl_16
    end
  end
  L8_2 = "Exporting one photo"
  ::lbl_16::
  L7_2.title = L8_2
  L5_2 = L5_2(L6_2, L7_2)
  L6_2 = {}
  L7_2 = "rgbOrigins"
  L8_2 = "settings"
  L9_2 = "autoWB"
  L10_2 = "roll"
  L11_2 = "rollName"
  L12_2 = "rollID"
  L13_2 = "nlpConverted"
  L6_2[1] = L7_2
  L6_2[2] = L8_2
  L6_2[3] = L9_2
  L6_2[4] = L10_2
  L6_2[5] = L11_2
  L6_2[6] = L12_2
  L6_2[7] = L13_2
  L7_2 = {}
  L8_2 = ipairs
  L9_2 = L6_2
  L8_2, L9_2, L10_2 = L8_2(L9_2)
  for L11_2, L12_2 in L8_2, L9_2, L10_2 do
    L7_2[L12_2] = true
  end
  L8_2 = {}
  L10_2 = L2_2
  L9_2 = L2_2.photosToExport
  L9_2, L10_2, L11_2 = L9_2(L10_2)
  for L12_2 in L9_2, L10_2, L11_2 do
    L13_2 = {}
    L14_2 = 1
    L15_2 = L15_1
    L15_2 = #L15_2
    L16_2 = 1
    for L17_2 = L14_2, L15_2, L16_2 do
      L18_2 = L15_1
      L18_2 = L18_2[L17_2]
      L18_2 = L18_2.id
      L19_2 = L7_2[L18_2]
      if not L19_2 then
        L20_2 = L12_2
        L19_2 = L12_2.getPropertyForPlugin
        L21_2 = _PLUGIN
        L22_2 = L18_2
        L19_2 = L19_2(L20_2, L21_2, L22_2)
        L13_2[L18_2] = L19_2
      end
    end
    L14_2 = table
    L14_2 = L14_2.insert
    L15_2 = L8_2
    L16_2 = L13_2
    L14_2(L15_2, L16_2)
  end
  L9_2 = A1_2.exportSession
  L10_2 = L9_2
  L9_2 = L9_2.renditions
  L9_2, L10_2, L11_2 = L9_2(L10_2)
  for L12_2, L13_2 in L9_2, L10_2, L11_2 do
    L15_2 = L5_2
    L14_2 = L5_2.setPortionComplete
    L16_2 = L12_2 - 1
    L16_2 = L16_2 / L4_2
    L14_2(L15_2, L16_2)
    L14_2 = L13_2.photo
    L16_2 = L13_2
    L15_2 = L13_2.waitForRender
    L15_2, L16_2 = L15_2(L16_2)
    L18_2 = L5_2
    L17_2 = L5_2.setPortionComplete
    L19_2 = L12_2 - 0.5
    L19_2 = L19_2 / L4_2
    L17_2(L18_2, L19_2)
    L18_2 = L5_2
    L17_2 = L5_2.isCanceled
    L17_2 = L17_2(L18_2)
    if L17_2 then
      break
    end
    if L15_2 then
      L17_2 = MAC_ENV
      if L17_2 then
        L17_2 = " -m -overwrite_original -E "
        if L17_2 then
          goto lbl_93
        end
      end
      L17_2 = " -charset Latin -m -overwrite_original -E "
      ::lbl_93::
      L18_2 = L10_1
      L18_2 = L18_2.generateCommands
      L19_2 = L14_2
      L20_2 = L8_2[L12_2]
      L21_2 = L3_2
      L22_2 = L12_2
      L18_2 = L18_2(L19_2, L20_2, L21_2, L22_2)
      L19_2 = L13_2.destinationPath
      L20_2 = L10_1
      L20_2 = L20_2.execute
      L21_2 = L17_2
      L22_2 = L18_2
      L23_2 = {}
      L24_2 = L19_2
      L23_2[1] = L24_2
      L20_2(L21_2, L22_2, L23_2)
      L20_2 = L8_1
      L20_2 = L20_2.sleep
      L21_2 = 0.1
      L20_2(L21_2)
      L20_2 = L3_2.reimportWithMetadata
      if L20_2 == true then
        L20_2 = L17_1
        L20_2 = L20_2.LR_reimportExportedPhoto
        if L20_2 == true then
          L20_2 = L22_1
          L21_2 = L20_2
          L20_2 = L20_2.withWriteAccessDo
          L22_2 = "reimport photo"
          
          function L23_2(A0_3)
            local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3
            L3_3 = L3_2
            L3_3 = L3_3.LR_reimport_stackWithOriginal
            if L3_3 == true then
              L3_3 = L13_2
              L1_3 = L3_3.photo
              L3_3 = L3_2
              L3_3 = L3_3.LR_reimport_stackWithOriginal_position
              L2_3 = L3_3 or L2_3
              if not L3_3 then
                L2_3 = "below"
              end
            end
            L3_3 = L22_1
            L4_3 = L3_3
            L3_3 = L3_3.addPhoto
            L5_3 = L13_2
            L5_3 = L5_3.destinationPath
            L6_3 = L1_3
            L7_3 = L2_3
            L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3)
            L4_3 = pairs
            L5_3 = L8_2
            L6_3 = L12_2
            L5_3 = L5_3[L6_3]
            L4_3, L5_3, L6_3 = L4_3(L5_3)
            for L7_3, L8_3 in L4_3, L5_3, L6_3 do
              if L8_3 then
                L10_3 = L3_3
                L9_3 = L3_3.setPropertyForPlugin
                L11_3 = _PLUGIN
                L12_3 = L7_3
                L13_3 = L8_3
                L9_3(L10_3, L11_3, L12_3, L13_3)
              end
            end
            L5_3 = L3_3
            L4_3 = L3_3.setPropertyForPlugin
            L6_3 = _PLUGIN
            L7_3 = "nlpConverted"
            L8_3 = "copy"
            L4_3(L5_3, L6_3, L7_3, L8_3)
          end
          
          L24_2 = {}
          L24_2.timeout = 100
          L20_2(L21_2, L22_2, L23_2, L24_2)
        end
      end
    end
  end
  L10_2 = L5_2
  L9_2 = L5_2.done
  L9_2(L10_2)
end

L18_1.processRenderedPhotos = L23_1
return L18_1
