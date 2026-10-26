local L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1

L1_1 = import
L2_1 = "LrFileUtils"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrPathUtils"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrLogger"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrTasks"
L4_1 = L4_1(L5_1)
L5_1 = require
L6_1 = "UtilV3"
L5_1 = L5_1(L6_1)
L6_1 = L3_1
L7_1 = "NegativeExport"
L6_1 = L6_1(L7_1)
L7_1 = _G
L7_1 = L7_1.dev
if L7_1 == true then
  L8_1 = L6_1
  L7_1 = L6_1.enable
  L9_1 = "logfile"
  L7_1(L8_1, L9_1)
else
  L8_1 = L6_1
  L7_1 = L6_1.disable
  L7_1(L8_1)
end
L7_1 = require
L8_1 = "MetadataDefinitionsV3"
L7_1 = L7_1(L8_1)
L8_1 = L7_1.metadataFieldsForPhotos
L9_1 = "\""
L10_1 = MAC_ENV
if L10_1 then
  L10_1 = "/mac/exiftool"
  if L10_1 then
    goto lbl_42
  end
end
L10_1 = "win\\exiftool.exe"
::lbl_42::
L11_1 = string
L11_1 = L11_1.format
L12_1 = "%s%s%s%s"
L13_1 = L9_1
L14_1 = _PLUGIN
L14_1 = L14_1.path
L15_1 = "/mac/exiftool"
L16_1 = L9_1
L11_1 = L11_1(L12_1, L13_1, L14_1, L15_1, L16_1)
L12_1 = WIN_ENV
if L12_1 then
  L12_1 = L5_1.cdCommand
  L13_1 = _PLUGIN
  L13_1 = L13_1.path
  L12_1 = L12_1(L13_1)
  L13_1 = " "
  L14_1 = L10_1
  L11_1 = L12_1 .. L13_1 .. L14_1
end

function L12_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "\n"
  L4_2 = " "
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  return A0_2
end

function L13_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2
  L4_2 = A1_2 or nil
  if not A1_2 then
    L4_2 = ""
  end
  L5_2 = A2_2 or L5_2
  if not A2_2 then
    L5_2 = ""
  end
  if A0_2 ~= nil then
    L6_2 = #A0_2
    if L6_2 ~= 0 and A0_2 ~= "none" then
      L6_2 = L12_1
      L7_2 = A0_2
      L6_2 = L6_2(L7_2)
      A0_2 = L6_2
      L6_2 = L4_2
      L7_2 = A0_2
      L8_2 = L5_2
      L6_2 = L6_2 .. L7_2 .. L8_2
      return L6_2
  end
  else
    L6_2 = A3_2 or L6_2
    if not A3_2 then
      L6_2 = ""
    end
    return L6_2
  end
end

L14_1 = {}
L15_1 = 1
L16_1 = #L8_1
L17_1 = 1
for L18_1 = L15_1, L16_1, L17_1 do
  L19_1 = L8_1[L18_1]
  L19_1 = L19_1.values
  if L19_1 ~= nil then
    L20_1 = #L19_1
    if 1 < L20_1 then
      L20_1 = L8_1[L18_1]
      L20_1 = L20_1.id
      L21_1 = {}
      L14_1[L20_1] = L21_1
      L21_1 = 1
      L22_1 = #L19_1
      L23_1 = 1
      for L24_1 = L21_1, L22_1, L23_1 do
        L25_1 = L19_1[L24_1]
        L25_1 = L25_1.value
        L26_1 = L14_1[L20_1]
        L27_1 = L19_1[L24_1]
        L27_1 = L27_1.title
        L26_1[L25_1] = L27_1
      end
    end
  end
end
L15_1 = {}
Exif = L15_1
L15_1 = Exif

function L16_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2
  L4_2 = MAC_ENV
  if L4_2 then
    L4_2 = "&#xa;"
    if L4_2 then
      goto lbl_8
    end
  end
  L4_2 = "&#xd;&#xa;"
  ::lbl_8::
  L5_2 = ""
  L6_2 = A2_2.appendCaption
  if L6_2 == true then
    L7_2 = A0_2
    L6_2 = A0_2.getFormattedMetadata
    L8_2 = "caption"
    L6_2 = L6_2(L7_2, L8_2)
    L5_2 = L6_2
  end
  L6_2 = A2_2.captionSeparator
  L7_2 = A1_2.nlpVersion
  if L7_2 then
    L7_2 = " v"
    L8_2 = A1_2.nlpVersion
    L7_2 = L7_2 .. L8_2
    if L7_2 then
      goto lbl_26
    end
  end
  L7_2 = ""
  ::lbl_26::
  L8_2 = ""
  L9_2 = A2_2.writeToExif
  if L9_2 == true then
    L9_2 = " -make="
    L10_2 = L9_1
    L11_2 = L13_1
    L12_2 = A1_2.nlpOriginalCameraMake
    L11_2 = L11_2(L12_2)
    L12_2 = L9_1
    L9_2 = L9_2 .. L10_2 .. L11_2 .. L12_2
    L10_2 = " -model="
    L11_2 = L9_1
    L12_2 = L13_1
    L13_2 = A1_2.nlpOriginalCameraModel
    L12_2 = L12_2(L13_2)
    L13_2 = L9_1
    L10_2 = L10_2 .. L11_2 .. L12_2 .. L13_2
    L11_2 = " -LensMake="
    L12_2 = L9_1
    L13_2 = L13_1
    L14_2 = A1_2.nlpOriginalLensMake
    L13_2 = L13_2(L14_2)
    L14_2 = L9_1
    L11_2 = L11_2 .. L12_2 .. L13_2 .. L14_2
    L12_2 = " -LensModel="
    L13_2 = L9_1
    L14_2 = L13_1
    L15_2 = A1_2.nlpOriginalLens
    L14_2 = L14_2(L15_2)
    L15_2 = L9_1
    L12_2 = L12_2 .. L13_2 .. L14_2 .. L15_2
    L13_2 = " -lens="
    L14_2 = L9_1
    L15_2 = L13_1
    L16_2 = A1_2.nlpOriginalLens
    L15_2 = L15_2(L16_2)
    L16_2 = L9_1
    L17_2 = " -LensInfo="
    L18_2 = L9_1
    L19_2 = L13_1
    L20_2 = A1_2.nlpOriginalLens
    L19_2 = L19_2(L20_2)
    L20_2 = L9_1
    L13_2 = L13_2 .. L14_2 .. L15_2 .. L16_2 .. L17_2 .. L18_2 .. L19_2 .. L20_2
    L14_2 = " -iso="
    L15_2 = L9_1
    L16_2 = L13_1
    L17_2 = A1_2.nlpFilmISO
    L16_2 = L16_2(L17_2)
    L17_2 = L9_1
    L14_2 = L14_2 .. L15_2 .. L16_2 .. L17_2
    L15_2 = " -ApertureValue="
    L16_2 = L9_1
    L17_2 = L13_1
    L18_2 = A1_2.nlpAperture
    L17_2 = L17_2(L18_2)
    L18_2 = L9_1
    L19_2 = " -FNumber="
    L20_2 = L9_1
    L21_2 = L13_1
    L22_2 = A1_2.nlpAperture
    L21_2 = L21_2(L22_2)
    L22_2 = L9_1
    L15_2 = L15_2 .. L16_2 .. L17_2 .. L18_2 .. L19_2 .. L20_2 .. L21_2 .. L22_2
    L16_2 = " -ShutterSpeedValue="
    L17_2 = L9_1
    L18_2 = L13_1
    L19_2 = A1_2.nlpShutterSpeed
    L18_2 = L18_2(L19_2)
    L19_2 = L9_1
    L20_2 = " -ExposureTime="
    L21_2 = L9_1
    L22_2 = L13_1
    L23_2 = A1_2.nlpShutterSpeed
    L22_2 = L22_2(L23_2)
    L23_2 = L9_1
    L16_2 = L16_2 .. L17_2 .. L18_2 .. L19_2 .. L20_2 .. L21_2 .. L22_2 .. L23_2
    L17_2 = " -FocalLength="
    L18_2 = L9_1
    L19_2 = L13_1
    L20_2 = A1_2.nlpFocalLength
    L19_2 = L19_2(L20_2)
    L20_2 = L9_1
    L17_2 = L17_2 .. L18_2 .. L19_2 .. L20_2
    L18_2 = " -ExposureCompensation="
    L19_2 = L13_1
    L20_2 = A1_2.nlpExposureBias
    L19_2 = L19_2(L20_2)
    L18_2 = L18_2 .. L19_2
    L19_2 = " -software="
    L20_2 = L9_1
    L21_2 = "Negative Lab Pro"
    L22_2 = L7_2
    L23_2 = L9_1
    L19_2 = L19_2 .. L20_2 .. L21_2 .. L22_2 .. L23_2
    L20_2 = L19_2
    L21_2 = " -CreatorTool="
    L22_2 = L9_1
    L23_2 = "Negative Lab Pro"
    L24_2 = L7_2
    L25_2 = L9_1
    L19_2 = L20_2 .. L21_2 .. L22_2 .. L23_2 .. L24_2 .. L25_2
    L20_2 = " -DateTimeOriginal="
    L21_2 = L9_1
    L22_2 = L13_1
    L23_2 = A1_2.nlpDateTaken
    L22_2 = L22_2(L23_2)
    L23_2 = " 00:00:00"
    L24_2 = L9_1
    L20_2 = L20_2 .. L21_2 .. L22_2 .. L23_2 .. L24_2
    L21_2 = ""
    L22_2 = A1_2.nlpFilmFormatSize
    if L22_2 then
      L22_2 = A1_2.nlpFocalLength
      if L22_2 then
        L22_2 = string
        L22_2 = L22_2.match
        L23_2 = A1_2.nlpFocalLength
        L24_2 = "%d+"
        L22_2 = L22_2(L23_2, L24_2)
        L23_2 = L5_1
        L23_2 = L23_2.round
        L24_2 = A1_2.nlpFilmFormatSize
        L24_2 = L22_2 * L24_2
        L23_2 = L23_2(L24_2)
        L24_2 = " -FocalLengthIn35mmFormat="
        L25_2 = L23_2
        L21_2 = L24_2 .. L25_2
    end
    else
      L21_2 = " -FocalLengthIn35mmFormat="
    end
    L22_2 = " -BrightnessValue= -SerialNumber= -LensSerialNumber= -MeteringMode= -Flash= -ExposureProgram= "
    L23_2 = L9_2
    L24_2 = L10_2
    L25_2 = L11_2
    L26_2 = L12_2
    L27_2 = L13_2
    L28_2 = L14_2
    L29_2 = L15_2
    L30_2 = L16_2
    L31_2 = L17_2
    L32_2 = L18_2
    L33_2 = L19_2
    L34_2 = L20_2
    L35_2 = L21_2
    L36_2 = L22_2
    L8_2 = L23_2 .. L24_2 .. L25_2 .. L26_2 .. L27_2 .. L28_2 .. L29_2 .. L30_2 .. L31_2 .. L32_2 .. L33_2 .. L34_2 .. L35_2 .. L36_2
  end
  L9_2 = ""
  L10_2 = A2_2.appendCaption
  if L10_2 == true then
    L10_2 = L13_1
    L11_2 = L5_2
    L10_2 = L10_2(L11_2)
    L9_2 = L10_2
  end
  L10_2 = ""
  L11_2 = A2_2.includeEquipmentCaption
  if L11_2 == true then
    L11_2 = L13_1
    L12_2 = A1_2.nlpOriginalCameraMake
    L13_2 = ""
    L14_2 = " "
    L11_2 = L11_2(L12_2, L13_2, L14_2)
    L12_2 = L13_1
    L13_2 = A1_2.nlpOriginalCameraModel
    L12_2 = L12_2(L13_2)
    L11_2 = L11_2 .. L12_2
    L12_2 = L13_1
    L13_2 = A1_2.nlpOriginalLensMake
    L14_2 = ""
    L15_2 = " "
    L12_2 = L12_2(L13_2, L14_2, L15_2)
    L13_2 = L13_1
    L14_2 = A1_2.nlpOriginalLens
    L13_2 = L13_2(L14_2)
    L12_2 = L12_2 .. L13_2
    L13_2 = L13_1
    L14_2 = A1_2.nlpFilmStock
    L15_2 = ""
    L16_2 = " "
    L13_2 = L13_2(L14_2, L15_2, L16_2)
    L14_2 = L13_1
    L15_2 = A1_2.nlpFilmISO
    L14_2 = L14_2(L15_2)
    L13_2 = L13_2 .. L14_2
    L14_2 = ""
    L15_2 = A1_2.nlpShotAtIso
    if L15_2 ~= nil then
      L15_2 = A1_2.nlpShotAtIso
      L16_2 = A1_2.nlpFilmISO
      if L15_2 ~= L16_2 then
        L15_2 = " @ "
        L16_2 = A1_2.nlpShotAtIso
        L14_2 = L15_2 .. L16_2
      end
    end
    L15_2 = A1_2.nlpFilmPushPull
    if L15_2 ~= nil then
      L15_2 = A1_2.nlpFilmPushPull
      if L15_2 ~= "none" then
        L15_2 = ", "
        L16_2 = A1_2.nlpFilmPushPull
        L17_2 = " dev"
        L15_2 = L15_2 .. L16_2 .. L17_2
        if L15_2 then
          goto lbl_238
        end
      end
    end
    L15_2 = ""
    ::lbl_238::
    L16_2 = L13_1
    L17_2 = L11_2
    L16_2 = L16_2(L17_2)
    L17_2 = L13_1
    L18_2 = L12_2
    L19_2 = L6_2
    L17_2 = L17_2(L18_2, L19_2)
    L18_2 = L13_1
    L19_2 = L13_2
    L20_2 = L6_2
    L18_2 = L18_2(L19_2, L20_2)
    L19_2 = L14_2
    L20_2 = L15_2
    L10_2 = L16_2 .. L17_2 .. L18_2 .. L19_2 .. L20_2
  end
  L11_2 = L13_1
  L12_2 = L7_2
  L13_2 = "Negative Lab Pro"
  L14_2 = ""
  L15_2 = "Negative Lab Pro"
  L11_2 = L11_2(L12_2, L13_2, L14_2, L15_2)
  L12_2 = ""
  L13_2 = A2_2.includeScanningCaption
  if L13_2 == true then
    L13_2 = A1_2.nlpScanMethod
    if L13_2 == "Digital Camera Scan" then
      L13_2 = "Digitized with "
      if L13_2 then
        goto lbl_269
      end
    end
    L13_2 = "Scanned with "
    ::lbl_269::
    L14_2 = L13_1
    L15_2 = A1_2.nlpScanEquipment
    L16_2 = L13_2
    L14_2 = L14_2(L15_2, L16_2)
    L15_2 = L13_1
    L16_2 = A1_2.nlpLightSource
    L17_2 = L6_2
    L15_2 = L15_2(L16_2, L17_2)
    L16_2 = L13_1
    L17_2 = A1_2.nlpFilmHolder
    L18_2 = L6_2
    L16_2 = L16_2(L17_2, L18_2)
    L12_2 = L14_2 .. L15_2 .. L16_2
  end
  L13_2 = ""
  L14_2 = A2_2.includeDevelopmentCaption
  if L14_2 == true then
    L14_2 = L13_1
    L15_2 = A1_2.nlpDevelopedAt
    L16_2 = ""
    L17_2 = " developed in"
    L18_2 = ""
    L14_2 = L14_2(L15_2, L16_2, L17_2, L18_2)
    L15_2 = L13_1
    L16_2 = A1_2.nlpDeveloper
    L17_2 = " "
    L15_2 = L15_2(L16_2, L17_2)
    L16_2 = L13_1
    L17_2 = A1_2.nlpDevDilution
    L18_2 = " "
    L16_2 = L16_2(L17_2, L18_2)
    L17_2 = L13_1
    L18_2 = A1_2.nlpDevTimeTemp
    L19_2 = L6_2
    L17_2 = L17_2(L18_2, L19_2)
    L18_2 = L13_1
    L19_2 = A1_2.nlpDevMethod
    L20_2 = L6_2
    L18_2 = L18_2(L19_2, L20_2)
    L13_2 = L14_2 .. L15_2 .. L16_2 .. L17_2 .. L18_2
  end
  L14_2 = nil
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = A1_2.nlpColor
  if L18_2 == "custom" then
    L18_2 = ""
    if L18_2 then
      goto lbl_320
      L14_2 = L18_2 or L14_2
    end
  end
  L18_2 = L14_1
  L18_2 = L18_2.nlpColor
  L19_2 = A1_2.nlpColor
  L14_2 = L18_2[L19_2]
  ::lbl_320::
  L18_2 = L14_1
  L18_2 = L18_2.nlpColorModel
  L19_2 = A1_2.nlpColorModel
  L15_2 = L18_2[L19_2]
  L18_2 = L14_1
  L18_2 = L18_2.nlpTones
  L19_2 = A1_2.nlpTones
  L16_2 = L18_2[L19_2]
  L18_2 = ""
  L19_2 = A2_2.includeNLPCaption
  if L19_2 == true then
    L19_2 = L13_1
    L20_2 = A1_2.nlpVersion
    L21_2 = "Negative Lab Pro v"
    L22_2 = ""
    L23_2 = ""
    L19_2 = L19_2(L20_2, L21_2, L22_2, L23_2)
    L20_2 = L13_1
    L21_2 = L15_2
    L22_2 = L6_2
    L23_2 = "Color Model: "
    L22_2 = L22_2 .. L23_2
    L23_2 = ""
    L24_2 = ""
    L20_2 = L20_2(L21_2, L22_2, L23_2, L24_2)
    L21_2 = L13_1
    L22_2 = A1_2.nlpPreSaturation
    L23_2 = L6_2
    L24_2 = "Pre-Sat: "
    L23_2 = L23_2 .. L24_2
    L24_2 = ""
    L25_2 = ""
    L21_2 = L21_2(L22_2, L23_2, L24_2, L25_2)
    L22_2 = L13_1
    L23_2 = L16_2
    L24_2 = L6_2
    L25_2 = "Tone Profile: "
    L24_2 = L24_2 .. L25_2
    L25_2 = ""
    L26_2 = ""
    L22_2 = L22_2(L23_2, L24_2, L25_2, L26_2)
    L23_2 = L13_1
    L24_2 = L14_2
    L25_2 = L6_2
    L26_2 = "WB: "
    L25_2 = L25_2 .. L26_2
    L26_2 = ""
    L27_2 = ""
    L23_2 = L23_2(L24_2, L25_2, L26_2, L27_2)
    L24_2 = L13_1
    L25_2 = A1_2.nlpLUT
    L26_2 = L6_2
    L27_2 = "LUT: "
    L26_2 = L26_2 .. L27_2
    L27_2 = ""
    L28_2 = ""
    L24_2 = L24_2(L25_2, L26_2, L27_2, L28_2)
    L18_2 = L19_2 .. L20_2 .. L21_2 .. L22_2 .. L23_2 .. L24_2
  end
  L19_2 = L9_1
  L20_2 = L13_1
  L21_2 = L9_2
  L22_2 = ""
  L23_2 = L4_2
  L24_2 = L4_2
  L23_2 = L23_2 .. L24_2
  L20_2 = L20_2(L21_2, L22_2, L23_2)
  L21_2 = L13_1
  L22_2 = L10_2
  L23_2 = ""
  L24_2 = L4_2
  L25_2 = L4_2
  L24_2 = L24_2 .. L25_2
  L21_2 = L21_2(L22_2, L23_2, L24_2)
  L22_2 = L13_1
  L23_2 = L12_2
  L24_2 = ""
  L25_2 = L4_2
  L26_2 = L4_2
  L25_2 = L25_2 .. L26_2
  L22_2 = L22_2(L23_2, L24_2, L25_2)
  L23_2 = L13_1
  L24_2 = L13_2
  L25_2 = ""
  L26_2 = L4_2
  L27_2 = L4_2
  L26_2 = L26_2 .. L27_2
  L23_2 = L23_2(L24_2, L25_2, L26_2)
  L24_2 = L13_1
  L25_2 = L18_2
  L24_2 = L24_2(L25_2)
  L25_2 = L9_1
  L19_2 = L19_2 .. L20_2 .. L21_2 .. L22_2 .. L23_2 .. L24_2 .. L25_2
  if A3_2 == 1 then
    L20_2 = A2_2.copyCaption
    if L20_2 == true then
      L20_2 = WIN_ENV
      if L20_2 then
        L20_2 = "clip"
        if L20_2 then
          goto lbl_425
        end
      end
      L20_2 = "pbcopy"
      ::lbl_425::
      L21_2 = "("
      L22_2 = L13_1
      L23_2 = L9_2
      L24_2 = "echo \""
      L25_2 = "\"\n"
      L22_2 = L22_2(L23_2, L24_2, L25_2)
      L23_2 = L13_1
      L24_2 = L10_2
      L25_2 = "echo \""
      L26_2 = "\"\n"
      L23_2 = L23_2(L24_2, L25_2, L26_2)
      L24_2 = L13_1
      L25_2 = L12_2
      L26_2 = "echo \""
      L27_2 = "\"\n"
      L24_2 = L24_2(L25_2, L26_2, L27_2)
      L25_2 = L13_1
      L26_2 = L13_2
      L27_2 = "echo \""
      L28_2 = "\"\n"
      L25_2 = L25_2(L26_2, L27_2, L28_2)
      L26_2 = ") | "
      L27_2 = L20_2
      L21_2 = L21_2 .. L22_2 .. L23_2 .. L24_2 .. L25_2 .. L26_2 .. L27_2
      L22_2 = L5_1
      L22_2 = L22_2.safeExecute
      L23_2 = L21_2
      L24_2 = "error_only"
      L25_2 = "copy caption to clipboard"
      L22_2 = L22_2(L23_2, L24_2, L25_2)
      if L22_2 ~= 0 then
      else
      end
    end
  end
  L20_2 = " -exif:ImageDescription="
  L21_2 = L19_2
  L22_2 = " -iptc:Caption-Abstract="
  L23_2 = L19_2
  L24_2 = " -xmp:Description="
  L25_2 = L19_2
  L20_2 = L20_2 .. L21_2 .. L22_2 .. L23_2 .. L24_2 .. L25_2
  L21_2 = L14_1
  L21_2 = L21_2.nlpFilmFormatSize
  L22_2 = A1_2.nlpFilmFormatSize
  L21_2 = L21_2[L22_2]
  if not L21_2 then
    L21_2 = ""
  end
  L22_2 = ""
  L23_2 = {}
  L24_2 = "Film"
  L25_2 = A1_2.nlpVersion
  if L25_2 then
    L25_2 = "Negative Lab Pro"
    if L25_2 then
      goto lbl_482
    end
  end
  L25_2 = ""
  ::lbl_482::
  L26_2 = L21_2 or L26_2
  if not L21_2 then
    L26_2 = ""
  end
  L27_2 = A1_2.nlpFilmStock
  if not L27_2 then
    L27_2 = ""
  end
  L28_2 = A1_2.nlpFilmISO
  if not L28_2 then
    L28_2 = ""
  end
  L29_2 = A1_2.nlpOriginalCameraMake
  if not L29_2 then
    L29_2 = ""
  end
  L30_2 = A1_2.nlpOriginalCameraModel
  if not L30_2 then
    L30_2 = ""
  end
  L31_2 = A1_2.nlpOriginalLensMake
  if not L31_2 then
    L31_2 = ""
  end
  L32_2 = A1_2.nlpOriginalLens
  if not L32_2 then
    L32_2 = ""
  end
  L33_2 = A1_2.nlpScanMethod
  if not L33_2 then
    L33_2 = ""
  end
  L34_2 = A1_2.nlpScanEquipment
  if not L34_2 then
    L34_2 = ""
  end
  L35_2 = A1_2.nlpLightSource
  if not L35_2 then
    L35_2 = ""
  end
  L36_2 = A1_2.nlpFilmHolder
  if not L36_2 then
    L36_2 = ""
  end
  L37_2 = A1_2.nlpDeveloper
  if not L37_2 then
    L37_2 = ""
  end
  L38_2 = A1_2.nlpDevDilution
  if not L38_2 then
    L38_2 = ""
  end
  L23_2[1] = L24_2
  L23_2[2] = L25_2
  L23_2[3] = L26_2
  L23_2[4] = L27_2
  L23_2[5] = L28_2
  L23_2[6] = L29_2
  L23_2[7] = L30_2
  L23_2[8] = L31_2
  L23_2[9] = L32_2
  L23_2[10] = L33_2
  L23_2[11] = L34_2
  L23_2[12] = L35_2
  L23_2[13] = L36_2
  L23_2[14] = L37_2
  L23_2[15] = L38_2
  L24_2 = A2_2.generateKeywords
  if L24_2 == true then
    L24_2 = 1
    L25_2 = #L23_2
    L26_2 = 1
    for L27_2 = L24_2, L25_2, L26_2 do
      L28_2 = L23_2[L27_2]
      if L28_2 ~= nil then
        L28_2 = L23_2[L27_2]
        if L28_2 ~= "" then
          L28_2 = L23_2[L27_2]
          if L28_2 ~= "none" then
            L28_2 = L22_2
            L29_2 = " -keywords-="
            L30_2 = L9_1
            L31_2 = L12_1
            L32_2 = L23_2[L27_2]
            L31_2 = L31_2(L32_2)
            L32_2 = L9_1
            L33_2 = " -keywords+="
            L34_2 = L9_1
            L35_2 = L12_1
            L36_2 = L23_2[L27_2]
            L35_2 = L35_2(L36_2)
            L36_2 = L9_1
            L22_2 = L28_2 .. L29_2 .. L30_2 .. L31_2 .. L32_2 .. L33_2 .. L34_2 .. L35_2 .. L36_2
          end
        end
      end
    end
  end
  L24_2 = L8_2
  L25_2 = L20_2
  L26_2 = L22_2
  L24_2 = L24_2 .. L25_2 .. L26_2
  return L24_2
end

L15_1.generateCommands = L16_1
L15_1 = Exif

function L16_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L3_2 = ""
  L4_2 = 1
  L5_2 = #A2_2
  L6_2 = 1
  for L7_2 = L4_2, L5_2, L6_2 do
    L8_2 = L3_2
    L9_2 = L9_1
    L10_2 = A2_2[L7_2]
    L11_2 = L9_1
    L3_2 = L8_2 .. L9_2 .. L10_2 .. L11_2
    L8_2 = #A2_2
    if L7_2 ~= L8_2 then
      L8_2 = L3_2
      L9_2 = " "
      L3_2 = L8_2 .. L9_2
    end
  end
  L4_2 = L11_1
  L5_2 = " "
  L6_2 = A0_2
  L7_2 = " "
  L8_2 = A1_2
  L9_2 = " "
  L10_2 = L3_2
  L4_2 = L4_2 .. L5_2 .. L6_2 .. L7_2 .. L8_2 .. L9_2 .. L10_2
  L5_2 = L5_1
  L5_2 = L5_2.safeExecute
  L6_2 = L4_2
  L7_2 = "error_only"
  L8_2 = "use exiftool to process metadata."
  L5_2 = L5_2(L6_2, L7_2, L8_2)
  if L5_2 ~= 0 then
  else
  end
end

L15_1.execute = L16_1
L15_1 = Exif
return L15_1
