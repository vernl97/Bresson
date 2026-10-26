local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1, L36_1, L37_1, L38_1, L39_1, L40_1, L41_1, L42_1, L43_1, L44_1, L45_1, L46_1, L47_1, L48_1, L49_1, L50_1, L51_1, L52_1, L53_1, L54_1, L55_1, L56_1, L57_1, L58_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrFunctionContext"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrDialogs"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrView"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrTasks"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrLogger"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrPathUtils"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrFileUtils"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrProgressScope"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrDevelopController"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrExportSession"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrPrefs"
L11_1 = L11_1(L12_1)
L11_1 = L11_1.prefsForPlugin
L11_1 = L11_1()
L12_1 = import
L13_1 = "LrColor"
L12_1 = L12_1(L13_1)
L13_1 = import
L14_1 = "LrApplication"
L13_1 = L13_1(L14_1)
L14_1 = require
L15_1 = "UtilV3"
L14_1 = L14_1(L15_1)
L15_1 = string
L15_1 = L15_1.format
L16_1 = require
L17_1 = "inspect"
L16_1 = L16_1(L17_1)
L17_1 = require
L18_1 = "NLPSessionV3"
L17_1 = L17_1(L18_1)
L18_1 = require
L19_1 = "NLPRolls"
L18_1 = L18_1(L19_1)
L19_1 = L5_1
L20_1 = "ImageProcessor"
L19_1 = L19_1(L20_1)
L20_1 = _G
L20_1 = L20_1.dev
if L20_1 == true then
  L21_1 = L19_1
  L20_1 = L19_1.enable
  L22_1 = "logfile"
  L20_1(L21_1, L22_1)
else
  L21_1 = L19_1
  L20_1 = L19_1.disable
  L20_1(L21_1)
end
L20_1 = ""
L21_1 = L6_1.getStandardFilePath
L22_1 = "temp"
L21_1 = L21_1(L22_1)
L22_1 = "\""
L23_1 = MAC_ENV
if L23_1 then
  L23_1 = "/mac/magick"
  if L23_1 then
    goto lbl_82
  end
end
L23_1 = "win\\convert.exe"
::lbl_82::
L24_1 = MAC_ENV
if L24_1 then
  L24_1 = "/mac/jpgicc"
  if L24_1 then
    goto lbl_89
  end
end
L24_1 = "win\\jpegicc.exe"
::lbl_89::
L25_1 = MAC_ENV
if L25_1 then
  L25_1 = "/mac/tificc"
  if L25_1 then
    goto lbl_96
  end
end
L25_1 = "win\\tificc.exe"
::lbl_96::
L26_1 = MAC_ENV
if L26_1 then
  L26_1 = "/profiles/MelissaRGB.icc"
  if L26_1 then
    goto lbl_103
  end
end
L26_1 = "profiles\\MelissaRGB.icc"
::lbl_103::
L27_1 = MAC_ENV
if L27_1 then
  L27_1 = "/profiles/sRGB.icc"
  if L27_1 then
    goto lbl_110
  end
end
L27_1 = "profiles\\sRGB.icc"
::lbl_110::
L28_1 = MAC_ENV
if L28_1 then
  L28_1 = "/profiles/ProPhoto.icc"
  if L28_1 then
    goto lbl_117
  end
end
L28_1 = "profiles\\ProPhoto.icc"
::lbl_117::
L29_1 = L15_1
L30_1 = "%s%s%s%s"
L31_1 = L22_1
L32_1 = _PLUGIN
L32_1 = L32_1.path
L33_1 = MAC_ENV
if L33_1 then
  L33_1 = "/mac/magick"
  if L33_1 then
    goto lbl_129
  end
end
L33_1 = "\\win\\convert.exe"
::lbl_129::
L34_1 = L22_1
L29_1 = L29_1(L30_1, L31_1, L32_1, L33_1, L34_1)
L30_1 = L15_1
L31_1 = "%s%s%s%s"
L32_1 = L22_1
L33_1 = _PLUGIN
L33_1 = L33_1.path
L34_1 = MAC_ENV
if L34_1 then
  L34_1 = "/mac/jpgicc"
  if L34_1 then
    goto lbl_143
  end
end
L34_1 = "\\win\\jpegicc.exe"
::lbl_143::
L35_1 = L22_1
L30_1 = L30_1(L31_1, L32_1, L33_1, L34_1, L35_1)
L31_1 = L15_1
L32_1 = "%s%s%s%s"
L33_1 = L22_1
L34_1 = _PLUGIN
L34_1 = L34_1.path
L35_1 = MAC_ENV
if L35_1 then
  L35_1 = "/mac/tificc"
  if L35_1 then
    goto lbl_157
  end
end
L35_1 = "\\win\\tificc.exe"
::lbl_157::
L36_1 = L22_1
L31_1 = L31_1(L32_1, L33_1, L34_1, L35_1, L36_1)
L32_1 = L15_1
L33_1 = "%s%s%s%s"
L34_1 = L22_1
L35_1 = _PLUGIN
L35_1 = L35_1.path
L36_1 = MAC_ENV
if L36_1 then
  L36_1 = "/profiles/MelissaRGB.icc"
  if L36_1 then
    goto lbl_171
  end
end
L36_1 = "\\profiles\\MelissaRGB.icc"
::lbl_171::
L37_1 = L22_1
L32_1 = L32_1(L33_1, L34_1, L35_1, L36_1, L37_1)
L33_1 = L15_1
L34_1 = "%s%s%s%s"
L35_1 = L22_1
L36_1 = _PLUGIN
L36_1 = L36_1.path
L37_1 = MAC_ENV
if L37_1 then
  L37_1 = "/profiles/sRGB.icc"
  if L37_1 then
    goto lbl_185
  end
end
L37_1 = "\\profiles\\sRGB.icc"
::lbl_185::
L38_1 = L22_1
L33_1 = L33_1(L34_1, L35_1, L36_1, L37_1, L38_1)
L34_1 = L15_1
L35_1 = "%s%s%s%s"
L36_1 = L22_1
L37_1 = _PLUGIN
L37_1 = L37_1.path
L38_1 = MAC_ENV
if L38_1 then
  L38_1 = "/profiles/ProPhoto.icc"
  if L38_1 then
    goto lbl_199
  end
end
L38_1 = "\\profiles\\ProPhoto.icc"
::lbl_199::
L39_1 = L22_1
L34_1 = L34_1(L35_1, L36_1, L37_1, L38_1, L39_1)
L35_1 = {}
L36_1 = import
L37_1 = "LrPrefs"
L36_1 = L36_1(L37_1)
L36_1 = L36_1.prefsForPlugin
L36_1 = L36_1()
L37_1 = L36_1.conversionLoggingEnabled
if not L37_1 then
  L37_1 = false
end
L38_1 = false
L39_1 = {}
L40_1 = {}
L40_1.black = 1.0E-4
L40_1.white = 1.0E-4
L39_1.positive = L40_1
L40_1 = {}
L40_1.black = 5.0E-4
L40_1.white = 1.0E-4
L39_1.negative = L40_1
L40_1 = {}
L40_1.black = 0
L40_1.white = 0
L39_1.editOnly = L40_1
L40_1 = {}
L41_1 = isAppleSilicon
if L41_1 then
  L41_1 = L15_1
  L42_1 = "%s%s%s%s"
  L43_1 = L22_1
  L44_1 = _PLUGIN
  L44_1 = L44_1.path
  L45_1 = "/mac/magick-arm64"
  L46_1 = L22_1
  L41_1 = L41_1(L42_1, L43_1, L44_1, L45_1, L46_1)
  L29_1 = L41_1
end
L41_1 = L13_1.versionTable
L41_1 = L41_1()
L42_1 = false
L43_1 = isAppleSilicon
if not L43_1 then
  L43_1 = MAC_ENV
  if L43_1 then
    L43_1 = L41_1.major
    if not (L43_1 < 13) then
      L43_1 = L41_1.major
      if L43_1 ~= 13 then
        goto lbl_257
      end
      L43_1 = L41_1.minor
      if not (L43_1 <= 4) then
        goto lbl_257
      end
    end
  end
end
L42_1 = true
::lbl_257::
L43_1 = _G
L43_1.testScenario = nil
L43_1 = _G
L43_1.testFailPhotoIndex = nil
L43_1 = {}
NLPImageProcessor = L43_1

function L43_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L0_2 = L40_1
  if L0_2 then
    L0_2 = L40_1
    L0_2 = #L0_2
    if 0 < L0_2 then
      L0_2 = ipairs
      L1_2 = L40_1
      L0_2, L1_2, L2_2 = L0_2(L1_2)
      for L3_2, L4_2 in L0_2, L1_2, L2_2 do
        L5_2 = L7_1
        L5_2 = L5_2.exists
        L6_2 = L4_2
        L5_2 = L5_2(L6_2)
        if L5_2 == "file" then
          L6_2 = pcall
          L7_2 = L7_1
          L7_2 = L7_2.delete
          L8_2 = L4_2
          L6_2, L7_2 = L6_2(L7_2, L8_2)
          if not L6_2 then
            L8_2 = L19_1
            L9_2 = L8_2
            L8_2 = L8_2.error
            L10_2 = string
            L10_2 = L10_2.format
            L11_2 = "Failed to delete temp file %s: %s"
            L12_2 = L4_2
            L13_2 = L7_2
            L10_2, L11_2, L12_2, L13_2 = L10_2(L11_2, L12_2, L13_2)
            L8_2(L9_2, L10_2, L11_2, L12_2, L13_2)
          else
          end
        elseif L5_2 == "folder" then
          L6_2 = L19_1
          L7_2 = L6_2
          L6_2 = L6_2.warn
          L8_2 = "Attempted to delete a folder listed in filesToDelete: "
          L9_2 = L4_2
          L6_2(L7_2, L8_2, L9_2)
        elseif L5_2 == false then
        end
      end
    end
  end
  L0_2 = {}
  L40_1 = L0_2
end

function L44_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = MAC_ENV
  if L1_2 then
    L1_2 = string
    L1_2 = L1_2.gsub
    L2_2 = A0_2
    L3_2 = "%("
    L4_2 = "\\("
    L1_2 = L1_2(L2_2, L3_2, L4_2)
    A0_2 = L1_2
    L1_2 = string
    L1_2 = L1_2.gsub
    L2_2 = A0_2
    L3_2 = "%)"
    L4_2 = "\\)"
    L1_2 = L1_2(L2_2, L3_2, L4_2)
    A0_2 = L1_2
    return A0_2
  end
  return A0_2
end

function L45_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L2_2 = A0_2
  L1_2 = A0_2.gsub
  L3_2 = "\\%("
  L4_2 = "("
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L2_2 = A0_2
  L1_2 = A0_2.gsub
  L3_2 = "\\%)"
  L4_2 = ")"
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  return A0_2
end

function L46_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  
  function L1_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3
    L2_3 = A0_3
    L1_3 = A0_3.gsub
    L3_3 = "[\r\n]+"
    L4_3 = " "
    L1_3 = L1_3(L2_3, L3_3, L4_3)
    A0_3 = L1_3
    L2_3 = A0_3
    L1_3 = A0_3.gsub
    L3_3 = "%s%s+"
    L4_3 = " "
    L1_3 = L1_3(L2_3, L3_3, L4_3)
    A0_3 = L1_3
    return A0_3
  end
  
  function L2_2(A0_3)
    local L1_3
    if A0_3 then
      L1_3 = ""
      return L1_3
    else
      L1_3 = " -threshold 0 "
      return L1_3
    end
  end
  
  function L3_2(A0_3)
    local L1_3
    if A0_3 == "positive" then
      L1_3 = "\\( -clone 0 -channel R -normalize -channel G -normalize -channel B -normalize +channel \\)"
      return L1_3
    else
      L1_3 = "\\( -clone 0 -channel R -normalize -channel G -normalize -channel B -normalize +channel -negate \\)"
      return L1_3
    end
  end
  
  function L4_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3, L16_3, L17_3, L18_3, L19_3, L20_3, L21_3, L22_3, L23_3, L24_3
    L1_3 = tostring
    L2_3 = A0_3.hueLowThreshold
    L1_3 = L1_3(L2_3)
    L2_3 = "%"
    L1_3 = L1_3 .. L2_3
    L2_3 = tostring
    L3_3 = A0_3.hueHighThreshold
    L2_3 = L2_3(L3_3)
    L3_3 = "%"
    L2_3 = L2_3 .. L3_3
    L3_3 = tostring
    L4_3 = A0_3.satThreshold
    L3_3 = L3_3(L4_3)
    L4_3 = "%"
    L3_3 = L3_3 .. L4_3
    L4_3 = L2_2
    L5_3 = A0_3.isWeightedLuminosity
    L4_3 = L4_3(L5_3)
    L5_3 = A0_3.invertHue
    if L5_3 == true then
      L5_3 = "-negate "
      if L5_3 then
        goto lbl_26
      end
    end
    L5_3 = ""
    ::lbl_26::
    L6_3 = [[
\( 
            mpr:original
        ]]
    L7_3 = A0_2
    L7_3 = L7_3.profilePath
    if L7_3 then
      L7_3 = A0_2
      L7_3 = L7_3.profilePath
      if L7_3 ~= "" then
        L7_3 = L6_3
        L8_3 = "            -profile "
        L9_3 = A0_2
        L9_3 = L9_3.profilePath
        L10_3 = "            "
        L6_3 = L7_3 .. L8_3 .. L9_3 .. L10_3
      end
    end
    L7_3 = L6_3
    L8_3 = "            "
    L9_3 = L3_2
    L10_3 = A0_3.imageType
    L9_3 = L9_3(L10_3)
    L10_3 = [[
            \( -clone 1 -colorspace HSB -channel red -separate +channel
               -black-threshold ]]
    L11_3 = L1_3
    L12_3 = "               -white-threshold "
    L13_3 = L2_3
    L14_3 = [[
               -fill black -opaque white -threshold 0
               ]]
    L15_3 = L5_3
    L16_3 = [[
            \)
            \( -clone 1 -colorspace HSB -channel green -negate -separate +channel
               -threshold ]]
    L17_3 = L3_3
    L18_3 = [[
            \)
            \( -clone 2-3 -compose Multiply -composite \)
            \( -clone 1 -colorspace HSL -channel blue -separate +channel
               -black-threshold ]]
    L19_3 = A0_3.luminosityLowThreshold
    L20_3 = [[
%
               -white-threshold ]]
    L21_3 = A0_3.luminosityHighThreshold
    L22_3 = [[
%
               -fill black -opaque white
               ]]
    L23_3 = L4_3
    L24_3 = [[
            \)
            \( -clone 4,5 -compose Multiply -composite \)
            -delete 0,2,3,4,5
            -compose CopyOpacity -composite
            -scale 1x1! -alpha off -colorspace HSB -depth 8
            -format "%[pixel:u]\n"
            -write info:
            +delete
        \)]]
    L6_3 = L7_3 .. L8_3 .. L9_3 .. L10_3 .. L11_3 .. L12_3 .. L13_3 .. L14_3 .. L15_3 .. L16_3 .. L17_3 .. L18_3 .. L19_3 .. L20_3 .. L21_3 .. L22_3 .. L23_3 .. L24_3
    L7_3 = L1_2
    L8_3 = L6_3
    L7_3 = L7_3(L8_3)
    L6_3 = L7_3
    return L6_3
  end
  
  L5_2 = {}
  L6_2 = table
  L6_2 = L6_2.insert
  L7_2 = L5_2
  L8_2 = A0_2.magickPath
  L6_2(L7_2, L8_2)
  L6_2 = table
  L6_2 = L6_2.insert
  L7_2 = L5_2
  L8_2 = "-quiet"
  L6_2(L7_2, L8_2)
  L6_2 = table
  L6_2 = L6_2.insert
  L7_2 = L5_2
  L8_2 = L14_1
  L8_2 = L8_2.quote
  L9_2 = A0_2.sourcePath
  L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2 = L8_2(L9_2)
  L6_2(L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2)
  L6_2 = A0_2.cropCommand
  if L6_2 then
    L6_2 = A0_2.cropCommand
    if L6_2 ~= "" then
      L6_2 = table
      L6_2 = L6_2.insert
      L7_2 = L5_2
      L8_2 = A0_2.cropCommand
      L6_2(L7_2, L8_2)
    end
  end
  L6_2 = A0_2.blurCommand
  if L6_2 then
    L6_2 = A0_2.blurCommand
    if L6_2 ~= "" then
      L6_2 = table
      L6_2 = L6_2.insert
      L7_2 = L5_2
      L8_2 = A0_2.blurCommand
      L6_2(L7_2, L8_2)
    end
  end
  L6_2 = table
  L6_2 = L6_2.insert
  L7_2 = L5_2
  L8_2 = "-write mpr:original +delete"
  L6_2(L7_2, L8_2)
  L6_2 = ipairs
  L7_2 = A0_2.whiteBalanceOptions
  L6_2, L7_2, L8_2 = L6_2(L7_2)
  for L9_2, L10_2 in L6_2, L7_2, L8_2 do
    L11_2 = table
    L11_2 = L11_2.insert
    L12_2 = L5_2
    L13_2 = L4_2
    L14_2 = L10_2
    L13_2, L14_2 = L13_2(L14_2)
    L11_2(L12_2, L13_2, L14_2)
  end
  L6_2 = [[
 mpr:original
       -intent Absolute
       -profile ]]
  L7_2 = A0_2.melissaRgbPath
  L8_2 = [[
       -strip -depth 8
       -set comment "blank comment"
       -write "]]
  L9_2 = A0_2.ppmOutputPath
  L10_2 = [[
"
    ]]
  L6_2 = L6_2 .. L7_2 .. L8_2 .. L9_2 .. L10_2
  L7_2 = table
  L7_2 = L7_2.insert
  L8_2 = L5_2
  L9_2 = L1_2
  L10_2 = L6_2
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2 = L9_2(L10_2)
  L7_2(L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2)
  L7_2 = table
  L7_2 = L7_2.insert
  L8_2 = L5_2
  L9_2 = "null:"
  L7_2(L8_2, L9_2)
  L7_2 = L1_2
  L8_2 = table
  L8_2 = L8_2.concat
  L9_2 = L5_2
  L10_2 = " "
  L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2 = L8_2(L9_2, L10_2)
  L7_2 = L7_2(L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2)
  L8_2 = MAC_ENV
  if not L8_2 then
    L8_2 = L45_1
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    L7_2 = L8_2
  end
  return L7_2
end

function L47_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L1_2 = ""
  L2_2 = nil
  L3_2 = MAC_ENV
  if L3_2 then
    L3_2 = L29_1
    L4_2 = " -quiet "
    L5_2 = L14_1
    L5_2 = L5_2.quote
    L6_2 = A0_2.photoPath
    L5_2 = L5_2(L6_2)
    L6_2 = " -profile "
    L7_2 = L34_1
    L8_2 = " "
    L9_2 = A0_2.cropCommand
    L10_2 = " -blur 0x1 "
    L1_2 = L3_2 .. L4_2 .. L5_2 .. L6_2 .. L7_2 .. L8_2 .. L9_2 .. L10_2
    L3_2 = L14_1
    L3_2 = L3_2.quote
    L4_2 = A0_2.infoPath
    L3_2 = L3_2(L4_2)
    L2_2 = L3_2
  else
    L3_2 = L14_1
    L3_2 = L3_2.cdCommand
    L4_2 = _PLUGIN
    L4_2 = L4_2.path
    L3_2 = L3_2(L4_2)
    L4_2 = " "
    L5_2 = L23_1
    L6_2 = " -quiet "
    L7_2 = L14_1
    L7_2 = L7_2.quote
    L8_2 = A0_2.photoPath
    L7_2 = L7_2(L8_2)
    L8_2 = " -profile "
    L9_2 = L14_1
    L9_2 = L9_2.quote
    L10_2 = L28_1
    L9_2 = L9_2(L10_2)
    L10_2 = " "
    L11_2 = A0_2.cropCommand
    L12_2 = " -blur 0x1 "
    L1_2 = L3_2 .. L4_2 .. L5_2 .. L6_2 .. L7_2 .. L8_2 .. L9_2 .. L10_2 .. L11_2 .. L12_2
    L3_2 = L14_1
    L3_2 = L3_2.quote
    L4_2 = A0_2.infoPath
    L3_2 = L3_2(L4_2)
    L2_2 = L3_2
  end
  L3_2 = ""
  L4_2 = A0_2.invertHue
  if L4_2 then
    L3_2 = "-negate "
  end
  L4_2 = ""
  L5_2 = A0_2.isWeightedLuminosity
  if L5_2 ~= true then
    L4_2 = " -threshold 0 "
  end
  L5_2 = A0_2.imageType
  if L5_2 == "positive" then
    L5_2 = L1_2
    L6_2 = L44_1
    L7_2 = " ( -clone 0 -channel R -normalize -channel G -normalize -channel B -normalize +channel ) "
    L6_2 = L6_2(L7_2)
    L1_2 = L5_2 .. L6_2
  else
    L5_2 = L1_2
    L6_2 = L44_1
    L7_2 = " ( -clone 0 -channel R -normalize -channel G -normalize -channel B -normalize +channel -negate ) "
    L6_2 = L6_2(L7_2)
    L1_2 = L5_2 .. L6_2
  end
  L5_2 = L1_2
  L6_2 = L44_1
  L7_2 = " ( -clone 1 -colorspace HSB -channel red -separate +channel -black-threshold "
  L8_2 = A0_2.hueLowThreshold
  L8_2 = L8_2 / 3.6
  L9_2 = "% -white-threshold "
  L10_2 = A0_2.hueHighThreshold
  L10_2 = L10_2 / 3.6
  L11_2 = "% -fill black -opaque white -threshold 0 "
  L12_2 = L3_2
  L13_2 = " ) "
  L7_2 = L7_2 .. L8_2 .. L9_2 .. L10_2 .. L11_2 .. L12_2 .. L13_2
  L6_2 = L6_2(L7_2)
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = L44_1
  L7_2 = " ( -clone 1 -colorspace HSB -channel green -negate -separate  +channel -threshold "
  L8_2 = A0_2.satThreshold
  L9_2 = "% ) "
  L7_2 = L7_2 .. L8_2 .. L9_2
  L6_2 = L6_2(L7_2)
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = L44_1
  L7_2 = " ( -clone 2-3 -compose Multiply -composite ) "
  L6_2 = L6_2(L7_2)
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = L44_1
  L7_2 = " ( -clone 1 -colorspace HSL -channel blue -separate +channel -black-threshold "
  L8_2 = A0_2.luminosityLowThreshold
  L9_2 = "% -white-threshold "
  L10_2 = A0_2.luminosityHighThreshold
  L11_2 = "%  -fill black -opaque white "
  L12_2 = L4_2
  L13_2 = " ) "
  L7_2 = L7_2 .. L8_2 .. L9_2 .. L10_2 .. L11_2 .. L12_2 .. L13_2
  L6_2 = L6_2(L7_2)
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = L44_1
  L7_2 = " ( -clone 4,5 -compose Multiply -composite ) "
  L6_2 = L6_2(L7_2)
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = " -delete 0,2,3,4,5 "
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = " -compose CopyOpacity -composite "
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = " -scale 1x1! -alpha off -colorspace HSB "
  L1_2 = L5_2 .. L6_2
  L5_2 = L1_2
  L6_2 = " -depth 8 -format \"%[pixel:u]\\n\" info: >> "
  L7_2 = L2_2
  L1_2 = L5_2 .. L6_2 .. L7_2
  return L1_2
end

function L48_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  if A0_2 then
    L1_2 = #A0_2
    if L1_2 == 3 then
      goto lbl_9
    end
  end
  L1_2 = nil
  L2_2 = "Invalid number of lines (expected 3)"
  do return L1_2, L2_2 end
  ::lbl_9::
  L1_2 = {}
  L2_2 = 1
  L3_2 = 3
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = string
    L6_2 = L6_2.match
    L7_2 = A0_2[L5_2]
    L8_2 = "^hsb%((%d*%.?%d+),(%d*%.?%d+)%%,(%d*%.?%d+)%%%)"
    L6_2, L7_2, L8_2 = L6_2(L7_2, L8_2)
    if not (L6_2 and L7_2) or not L8_2 then
      L9_2 = nil
      L10_2 = "Invalid format in line %d"
      L11_2 = L10_2
      L10_2 = L10_2.format
      L12_2 = L5_2
      L10_2, L11_2, L12_2 = L10_2(L11_2, L12_2)
      return L9_2, L10_2, L11_2, L12_2
    end
    L9_2 = {}
    L10_2 = tonumber
    L11_2 = L6_2
    L10_2 = L10_2(L11_2)
    L9_2.h = L10_2
    L10_2 = tonumber
    L11_2 = L7_2
    L10_2 = L10_2(L11_2)
    L9_2.s = L10_2
    L10_2 = tonumber
    L11_2 = L8_2
    L10_2 = L10_2(L11_2)
    L9_2.b = L10_2
    L1_2[L5_2] = L9_2
    L9_2 = L1_2[L5_2]
    L9_2 = L9_2.h
    if L9_2 then
      L9_2 = L1_2[L5_2]
      L9_2 = L9_2.s
      if L9_2 then
        L9_2 = L1_2[L5_2]
        L9_2 = L9_2.b
        if L9_2 then
          goto lbl_63
        end
      end
    end
    L9_2 = nil
    L10_2 = "Non-numeric value in line %d"
    L11_2 = L10_2
    L10_2 = L10_2.format
    L12_2 = L5_2
    L10_2, L11_2, L12_2 = L10_2(L11_2, L12_2)
    do return L9_2, L10_2, L11_2, L12_2 end
    ::lbl_63::
  end
  L2_2 = {}
  L3_2 = L1_2[1]
  L3_2 = L3_2.h
  L2_2.h1 = L3_2
  L3_2 = L1_2[1]
  L3_2 = L3_2.s
  L2_2.s1 = L3_2
  L3_2 = L1_2[1]
  L3_2 = L3_2.b
  L2_2.b1 = L3_2
  L3_2 = L1_2[2]
  L3_2 = L3_2.h
  L2_2.h2 = L3_2
  L3_2 = L1_2[2]
  L3_2 = L3_2.s
  L2_2.s2 = L3_2
  L3_2 = L1_2[2]
  L3_2 = L3_2.b
  L2_2.b2 = L3_2
  L3_2 = L1_2[3]
  L3_2 = L3_2.h
  L2_2.h3 = L3_2
  L3_2 = L1_2[3]
  L3_2 = L3_2.s
  L2_2.s3 = L3_2
  L3_2 = L1_2[3]
  L3_2 = L3_2.b
  L2_2.b3 = L3_2
  return L2_2
end

function L49_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2
  L5_2 = L7_1
  L5_2 = L5_2.isReadable
  L6_2 = A0_2
  L5_2 = L5_2(L6_2)
  if not L5_2 then
    L5_2 = string
    L5_2 = L5_2.format
    L6_2 = [[


ERROR: Could not read photo for analysis:
%s

Make sure the photo is available (e.g., external drive connected).]]
    L7_2 = A0_2
    L5_2 = L5_2(L6_2, L7_2)
    L6_2 = L19_1
    L7_2 = L6_2
    L6_2 = L6_2.error
    L8_2 = L5_2
    L6_2(L7_2, L8_2)
    L6_2 = L35_1
    L7_2 = L35_1
    L7_2 = #L7_2
    L7_2 = L7_2 + 1
    L6_2[L7_2] = L5_2
    L6_2 = nil
    L7_2 = nil
    L8_2 = true
    return L6_2, L7_2, L8_2
  end
  L5_2 = L6_1
  L5_2 = L5_2.child
  L6_2 = L21_1
  L7_2 = A1_2
  L8_2 = ".ppm"
  L7_2 = L7_2 .. L8_2
  L5_2 = L5_2(L6_2, L7_2)
  L6_2 = L6_1
  L6_2 = L6_2.child
  L7_2 = L21_1
  L8_2 = A1_2
  L9_2 = ".txt"
  L8_2 = L8_2 .. L9_2
  L6_2 = L6_2(L7_2, L8_2)
  L7_2 = A2_2.filmBorder
  L7_2 = L7_2 * 2
  L7_2 = 100 - L7_2
  L8_2 = ""
  L9_2 = A2_2.previewBuffer
  L9_2 = not L9_2
  if L9_2 == true and L7_2 < 100 then
    L9_2 = "-gravity Center -crop "
    L10_2 = L7_2
    L11_2 = "x"
    L12_2 = L7_2
    L13_2 = "%+0+0 +repage"
    L8_2 = L9_2 .. L10_2 .. L11_2 .. L12_2 .. L13_2
  end
  L9_2 = " -blur 0x2 "
  L10_2 = nil
  L11_2 = nil
  L12_2 = nil
  L13_2 = MAC_ENV
  if L13_2 then
    L13_2 = isAppleSilicon
    if L13_2 then
      L13_2 = L14_1
      L13_2 = L13_2.quote
      L14_2 = _PLUGIN
      L14_2 = L14_2.path
      L15_2 = "/mac/magick-arm64"
      L14_2 = L14_2 .. L15_2
      L13_2 = L13_2(L14_2)
      if L13_2 then
        goto lbl_79
        L10_2 = L13_2 or L10_2
      end
    end
    L13_2 = L14_1
    L13_2 = L13_2.quote
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L15_2 = "/mac/magick"
    L14_2 = L14_2 .. L15_2
    L13_2 = L13_2(L14_2)
    L10_2 = L13_2
    ::lbl_79::
    L13_2 = L14_1
    L13_2 = L13_2.quote
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L15_2 = "/profiles/ProPhoto.icc"
    L14_2 = L14_2 .. L15_2
    L13_2 = L13_2(L14_2)
    L11_2 = L13_2
    L13_2 = L14_1
    L13_2 = L13_2.quote
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L15_2 = "/profiles/MelissaRGB.icc"
    L14_2 = L14_2 .. L15_2
    L13_2 = L13_2(L14_2)
    L12_2 = L13_2
  else
    L13_2 = L14_1
    L13_2 = L13_2.cdCommand
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L13_2 = L13_2(L14_2)
    L14_2 = " "
    L15_2 = L23_1
    L10_2 = L13_2 .. L14_2 .. L15_2
    L13_2 = L14_1
    L13_2 = L13_2.quote
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L15_2 = "\\profiles\\ProPhoto.icc"
    L14_2 = L14_2 .. L15_2
    L13_2 = L13_2(L14_2)
    L11_2 = L13_2
    L13_2 = L14_1
    L13_2 = L13_2.quote
    L14_2 = _PLUGIN
    L14_2 = L14_2.path
    L15_2 = "\\profiles\\MelissaRGB.icc"
    L14_2 = L14_2 .. L15_2
    L13_2 = L13_2(L14_2)
    L12_2 = L13_2
  end
  L13_2 = {}
  L13_2.magickPath = L10_2
  L13_2.sourcePath = A0_2
  L13_2.profilePath = L11_2
  L13_2.outputInfoPath = L6_2
  L13_2.blurCommand = L9_2
  L13_2.cropCommand = L8_2
  L13_2.melissaRgbPath = L12_2
  L13_2.ppmOutputPath = L5_2
  L14_2 = {}
  L15_2 = {}
  L16_2 = A2_2.imageType
  L15_2.imageType = L16_2
  L15_2.hueLowThreshold = 0
  L15_2.hueHighThreshold = 100
  L15_2.invertHue = false
  L15_2.satThreshold = 70
  L15_2.luminosityLowThreshold = 0
  L15_2.luminosityHighThreshold = 100
  L15_2.isWeightedLuminosity = false
  L16_2 = {}
  L17_2 = A2_2.imageType
  L16_2.imageType = L17_2
  L16_2.hueLowThreshold = 40.2778
  L16_2.hueHighThreshold = 75
  L16_2.invertHue = false
  L16_2.satThreshold = 60
  L16_2.luminosityLowThreshold = 25
  L16_2.luminosityHighThreshold = 75
  L16_2.isWeightedLuminosity = false
  L17_2 = {}
  L18_2 = A2_2.imageType
  L17_2.imageType = L18_2
  L17_2.hueLowThreshold = 25
  L17_2.hueHighThreshold = 75
  L17_2.invertHue = true
  L17_2.satThreshold = 70
  L17_2.luminosityLowThreshold = 25
  L17_2.luminosityHighThreshold = 75
  L17_2.isWeightedLuminosity = false
  L14_2[1] = L15_2
  L14_2[2] = L16_2
  L14_2[3] = L17_2
  L13_2.whiteBalanceOptions = L14_2
  L14_2 = L46_1
  L15_2 = L13_2
  L14_2 = L14_2(L15_2)
  L15_2 = 4
  L16_2 = 0
  L17_2 = false
  L18_2 = nil
  L19_2 = nil
  L20_2 = nil
  L21_2 = nil
  L22_2 = nil
  L23_2 = nil
  L24_2 = nil
  L25_2 = nil
  L26_2 = false
  L27_2 = nil
  while L15_2 > L16_2 and not L17_2 do
    L27_2 = false
    L28_2 = pcall
    L29_2 = L7_1
    L29_2 = L29_2.delete
    L30_2 = L5_2
    L28_2(L29_2, L30_2)
    L28_2 = pcall
    L29_2 = L7_1
    L29_2 = L29_2.delete
    L30_2 = L6_2
    L28_2(L29_2, L30_2)
    L28_2 = L4_1
    L28_2 = L28_2.sleep
    L29_2 = 0.01
    L28_2(L29_2)
    L28_2 = L14_1
    L28_2 = L28_2.safeExecute
    L29_2 = L14_2
    L30_2 = "separate"
    L31_2 = "analyze photo "
    L32_2 = A3_2
    L31_2 = L31_2 .. L32_2
    L28_2, L29_2, L30_2 = L28_2(L29_2, L30_2, L31_2)
    L20_2 = L30_2
    L19_2 = L29_2
    L18_2 = L28_2
    L28_2 = _G
    L28_2 = L28_2.testScenario
    if L28_2 then
      L28_2 = _G
      L28_2 = L28_2.testFailPhotoIndex
      if L28_2 then
        L28_2 = _G
        L28_2 = L28_2.testFailPhotoIndex
        if A3_2 == L28_2 then
          L28_2 = L19_1
          L29_2 = L28_2
          L28_2 = L28_2.warn
          L30_2 = string
          L30_2 = L30_2.format
          L31_2 = "!!! TEST SCENARIO '%s' INJECTED FOR PHOTO %d !!!"
          L32_2 = _G
          L32_2 = L32_2.testScenario
          L33_2 = A3_2
          L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2 = L30_2(L31_2, L32_2, L33_2)
          L28_2(L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2)
          L28_2 = _G
          L28_2 = L28_2.testScenario
          if L28_2 == "fail_magick" then
            L18_2 = 1
            if not L20_2 then
              L20_2 = "Simulated ImageMagick Failure"
            end
            L19_2 = ""
            L28_2 = pcall
            L29_2 = L7_1
            L29_2 = L29_2.delete
            L30_2 = L5_2
            L28_2(L29_2, L30_2)
            L28_2 = L19_1
            L29_2 = L28_2
            L28_2 = L28_2.warn
            L30_2 = " > Simulating non-zero exit code and PPM deletion."
            L28_2(L29_2, L30_2)
          else
            L28_2 = _G
            L28_2 = L28_2.testScenario
            if L28_2 == "fail_wb_parse" then
              L18_2 = 0
              L19_2 = [[
invalid
hsb
data]]
              L28_2 = L19_1
              L29_2 = L28_2
              L28_2 = L28_2.warn
              L30_2 = " > Simulating invalid stdout for WB parsing."
              L28_2(L29_2, L30_2)
            else
              L28_2 = _G
              L28_2 = L28_2.testScenario
              if L28_2 == "fail_ppm_open" then
                L18_2 = 0
                L28_2 = pcall
                L29_2 = L7_1
                L29_2 = L29_2.delete
                L30_2 = L5_2
                L28_2(L29_2, L30_2)
                L28_2 = L19_1
                L29_2 = L28_2
                L28_2 = L28_2.warn
                L30_2 = " > Simulating PPM file deletion before open attempt."
                L28_2(L29_2, L30_2)
              else
                L28_2 = _G
                L28_2 = L28_2.testScenario
                if L28_2 == "fail_ppm_read" then
                  L18_2 = 0
                  L28_2 = pcall
                  
                  function L29_2()
                    local L0_3, L1_3, L2_3, L3_3
                    L0_3 = io
                    L0_3 = L0_3.open
                    L1_3 = L5_2
                    L2_3 = "w"
                    L0_3 = L0_3(L1_3, L2_3)
                    if L0_3 then
                      L2_3 = L0_3
                      L1_3 = L0_3.write
                      L3_3 = [[
P6
#invalid
10 10
255
]]
                      L1_3(L2_3, L3_3)
                      L2_3 = L0_3
                      L1_3 = L0_3.close
                      L1_3(L2_3)
                      L1_3 = L19_1
                      L2_3 = L1_3
                      L1_3 = L1_3.warn
                      L3_3 = " > Simulating PPM file corruption/read error by writing invalid content."
                      L1_3(L2_3, L3_3)
                    else
                      L1_3 = L19_1
                      L2_3 = L1_3
                      L1_3 = L1_3.error
                      L3_3 = " > Could not open PPM file to simulate corruption."
                      L1_3(L2_3, L3_3)
                    end
                  end
                  
                  L28_2, L29_2 = L28_2(L29_2)
                  if not L28_2 then
                    L30_2 = L19_1
                    L31_2 = L30_2
                    L30_2 = L30_2.error
                    L32_2 = " > Error simulating PPM corruption: "
                    L33_2 = L29_2
                    L30_2(L31_2, L32_2, L33_2)
                  end
                end
              end
            end
          end
        end
      end
    end
    if L20_2 and L20_2 ~= "" then
      L28_2 = L19_1
      L29_2 = L28_2
      L28_2 = L28_2.warn
      L30_2 = "Stderr: "
      L31_2 = L20_2
      L28_2(L29_2, L30_2, L31_2)
    end
    if L18_2 == 0 then
      L28_2 = {}
      L21_2 = L28_2
      L22_2 = nil
      L23_2 = nil
      L28_2 = type
      L29_2 = L19_2
      L28_2 = L28_2(L29_2)
      if L28_2 == "string" then
        L29_2 = L19_2
        L28_2 = L19_2.gmatch
        L30_2 = "([^\r\n]+)"
        L28_2, L29_2, L30_2 = L28_2(L29_2, L30_2)
        for L31_2 in L28_2, L29_2, L30_2 do
          L32_2 = table
          L32_2 = L32_2.insert
          L33_2 = L21_2
          L34_2 = L31_2
          L32_2(L33_2, L34_2)
        end
      else
        L23_2 = "stdout was not a string"
      end
      if not L23_2 then
        L28_2 = #L21_2
        if L28_2 ~= 3 then
          L28_2 = "Expected 3 lines, got "
          L29_2 = #L21_2
          L23_2 = L28_2 .. L29_2
        end
      end
      if not L23_2 then
        L28_2 = L48_1
        L29_2 = L21_2
        L28_2, L29_2 = L28_2(L29_2)
        L23_2 = L29_2
        L22_2 = L28_2
      end
      if L22_2 then
        L28_2 = io
        L28_2 = L28_2.open
        L29_2 = L5_2
        L30_2 = "rb"
        L28_2, L29_2 = L28_2(L29_2, L30_2)
        L25_2 = L29_2
        L24_2 = L28_2
        if L24_2 then
          L17_2 = true
          L29_2 = L24_2
          L28_2 = L24_2.close
          L28_2(L29_2)
          L28_2 = L40_1
          L29_2 = L40_1
          L29_2 = #L29_2
          L29_2 = L29_2 + 1
          L28_2[L29_2] = L5_2
          L28_2 = L7_1
          L28_2 = L28_2.exists
          L29_2 = L6_2
          L28_2 = L28_2(L29_2)
          if L28_2 then
            L28_2 = L40_1
            L29_2 = L40_1
            L29_2 = #L29_2
            L29_2 = L29_2 + 1
            L28_2[L29_2] = L6_2
          end
          L27_2 = true
        else
          L28_2 = L19_1
          L29_2 = L28_2
          L28_2 = L28_2.warn
          L30_2 = string
          L30_2 = L30_2.format
          L31_2 = "Failed to open PPM file on attempt %d for photo %d (%s)."
          L32_2 = L16_2 + 1
          L33_2 = A3_2
          L34_2 = L25_2 or L34_2
          if not L25_2 then
            L34_2 = "unknown I/O error"
          end
          L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2 = L30_2(L31_2, L32_2, L33_2, L34_2)
          L28_2(L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2)
          if L24_2 then
            L28_2 = pcall
            
            function L29_2()
              local L0_3, L1_3
              L0_3 = L24_2
              L1_3 = L0_3
              L0_3 = L0_3.close
              L0_3(L1_3)
            end
            
            L28_2(L29_2)
          end
        end
      else
        L28_2 = L19_1
        L29_2 = L28_2
        L28_2 = L28_2.warn
        L30_2 = string
        L30_2 = L30_2.format
        L31_2 = "Failed to parse WB data on attempt %d for photo %d (%s)."
        L32_2 = L16_2 + 1
        L33_2 = A3_2
        L34_2 = L23_2 or L34_2
        if not L23_2 then
          L34_2 = "unknown parse error"
        end
        L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2 = L30_2(L31_2, L32_2, L33_2, L34_2)
        L28_2(L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2)
      end
    else
      L28_2 = L19_1
      L29_2 = L28_2
      L28_2 = L28_2.warn
      L30_2 = string
      L30_2 = L30_2.format
      L31_2 = "ImageMagick failed on attempt %d for photo %d (Exit Code: %d)."
      L32_2 = L16_2 + 1
      L33_2 = A3_2
      L34_2 = L18_2
      L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2 = L30_2(L31_2, L32_2, L33_2, L34_2)
      L28_2(L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2)
    end
    if not L27_2 then
      L16_2 = L16_2 + 1
      if L15_2 > L16_2 then
        L28_2 = L4_1
        L28_2 = L28_2.sleep
        L29_2 = 0.5 * L16_2
        L28_2(L29_2)
      end
    end
  end
  L28_2 = {}
  L29_2 = nil
  if L17_2 then
    L30_2 = {}
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.h1
    L31_2 = L31_2(L32_2)
    L30_2.h = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.s1
    L31_2 = L31_2(L32_2)
    L30_2.s = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.b1
    L31_2 = L31_2(L32_2)
    L30_2.b = L31_2
    L28_2.smartColorNeutral = L30_2
    L30_2 = {}
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.h2
    L31_2 = L31_2(L32_2)
    L30_2.h = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.s2
    L31_2 = L31_2(L32_2)
    L30_2.s = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.b2
    L31_2 = L31_2(L32_2)
    L30_2.b = L31_2
    L28_2.smartColorWarming = L30_2
    L30_2 = {}
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.h3
    L31_2 = L31_2(L32_2)
    L30_2.h = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.s3
    L31_2 = L31_2(L32_2)
    L30_2.s = L31_2
    L31_2 = L14_1
    L31_2 = L31_2.round
    L32_2 = L22_2.b3
    L31_2 = L31_2(L32_2)
    L30_2.b = L31_2
    L28_2.smartColorCooling = L30_2
    L26_2 = false
  else
    L30_2 = L19_1
    L31_2 = L30_2
    L30_2 = L30_2.error
    L32_2 = string
    L32_2 = L32_2.format
    L33_2 = "Analysis failed after %d attempts for photo %d."
    L34_2 = L15_2
    L35_2 = A3_2
    L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2 = L32_2(L33_2, L34_2, L35_2)
    L30_2(L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2)
    L30_2 = io
    L30_2 = L30_2.open
    L31_2 = L5_2
    L32_2 = "rb"
    L30_2, L31_2 = L30_2(L31_2, L32_2)
    L25_2 = L31_2
    L24_2 = L30_2
    if L24_2 then
      L30_2 = L19_1
      L31_2 = L30_2
      L30_2 = L30_2.warn
      L32_2 = "Partial success for photo "
      L33_2 = A3_2
      L34_2 = ": PPM readable, but WB analysis failed. Using dummy WB data."
      L32_2 = L32_2 .. L33_2 .. L34_2
      L30_2(L31_2, L32_2)
      L31_2 = L24_2
      L30_2 = L24_2.close
      L30_2(L31_2)
      L30_2 = L40_1
      L31_2 = L40_1
      L31_2 = #L31_2
      L31_2 = L31_2 + 1
      L30_2[L31_2] = L5_2
      L30_2 = L35_1
      L31_2 = L35_1
      L31_2 = #L31_2
      L31_2 = L31_2 + 1
      L32_2 = string
      L32_2 = L32_2.format
      L33_2 = [[


White Balance analysis failed for photo %d of %d after %d attempts. Proceeding with dummy WB data. Last Error: %s]]
      L34_2 = A3_2
      L35_2 = A4_2
      L36_2 = L15_2
      L37_2 = L23_2 or L37_2
      if not L23_2 then
        L37_2 = "ImageMagick error "
        L38_2 = L18_2
        L39_2 = ": "
        L40_2 = L20_2 or L40_2
        if not L20_2 then
          L40_2 = "Unknown"
        end
        L37_2 = L37_2 .. L38_2 .. L39_2 .. L40_2
      end
      L32_2 = L32_2(L33_2, L34_2, L35_2, L36_2, L37_2)
      L30_2[L31_2] = L32_2
      L30_2 = {}
      L30_2.h = 0
      L30_2.s = 0
      L30_2.b = 0
      L28_2.smartColorNeutral = L30_2
      L30_2 = {}
      L30_2.h = 0
      L30_2.s = 0
      L30_2.b = 0
      L28_2.smartColorWarming = L30_2
      L30_2 = {}
      L30_2.h = 0
      L30_2.s = 0
      L30_2.b = 0
      L28_2.smartColorCooling = L30_2
      L26_2 = true
    else
      L30_2 = L19_1
      L31_2 = L30_2
      L30_2 = L30_2.error
      L32_2 = "Total failure for photo "
      L33_2 = A3_2
      L34_2 = ": PPM file not readable. "
      L35_2 = L25_2 or L35_2
      if not L25_2 then
        L35_2 = ""
      end
      L32_2 = L32_2 .. L33_2 .. L34_2 .. L35_2
      L30_2(L31_2, L32_2)
      L30_2 = L35_1
      L31_2 = L35_1
      L31_2 = #L31_2
      L31_2 = L31_2 + 1
      L32_2 = string
      L32_2 = L32_2.format
      L33_2 = [[


Critical analysis failure for photo %d of %d after %d attempts. Cannot read image data (PPM). Last IM Exit: %d. Last Error: %s]]
      L34_2 = A3_2
      L35_2 = A4_2
      L36_2 = L15_2
      L37_2 = L18_2 or L37_2
      if not L18_2 then
        L37_2 = -1
      end
      L38_2 = L20_2 or L38_2
      L38_2 = L25_2 or L38_2
      if not L20_2 and not L25_2 then
        L38_2 = "Unknown"
      end
      L32_2 = L32_2(L33_2, L34_2, L35_2, L36_2, L37_2, L38_2)
      L30_2[L31_2] = L32_2
      L30_2 = L7_1
      L30_2 = L30_2.exists
      L31_2 = L5_2
      L30_2 = L30_2(L31_2)
      if L30_2 then
        L30_2 = L40_1
        L31_2 = L40_1
        L31_2 = #L31_2
        L31_2 = L31_2 + 1
        L30_2[L31_2] = L5_2
      end
      L30_2 = L7_1
      L30_2 = L30_2.exists
      L31_2 = L6_2
      L30_2 = L30_2(L31_2)
      if L30_2 then
        L30_2 = L40_1
        L31_2 = L40_1
        L31_2 = #L31_2
        L31_2 = L31_2 + 1
        L30_2[L31_2] = L6_2
      end
      L30_2 = nil
      L31_2 = nil
      L32_2 = true
      return L30_2, L31_2, L32_2
    end
  end
  L30_2 = io
  L30_2 = L30_2.open
  L31_2 = L5_2
  L32_2 = "rb"
  L30_2, L31_2 = L30_2(L31_2, L32_2)
  L25_2 = L31_2
  L24_2 = L30_2
  if not L24_2 then
    L30_2 = L19_1
    L31_2 = L30_2
    L30_2 = L30_2.error
    L32_2 = "Consistency error: Failed to re-open PPM file after successful check for photo "
    L33_2 = A3_2
    L34_2 = ": "
    L35_2 = L25_2 or L35_2
    if not L25_2 then
      L35_2 = "Unknown error"
    end
    L32_2 = L32_2 .. L33_2 .. L34_2 .. L35_2
    L30_2(L31_2, L32_2)
    L30_2 = L35_1
    L31_2 = L35_1
    L31_2 = #L31_2
    L31_2 = L31_2 + 1
    L32_2 = [[


Internal Error: Could not read PPM data for photo ]]
    L33_2 = A3_2
    L34_2 = " even after initial checks passed."
    L32_2 = L32_2 .. L33_2 .. L34_2
    L30_2[L31_2] = L32_2
    L30_2 = L7_1
    L30_2 = L30_2.exists
    L31_2 = L5_2
    L30_2 = L30_2(L31_2)
    if L30_2 then
      L30_2 = L40_1
      L31_2 = L40_1
      L31_2 = #L31_2
      L31_2 = L31_2 + 1
      L30_2[L31_2] = L5_2
    end
    L30_2 = L7_1
    L30_2 = L30_2.exists
    L31_2 = L6_2
    L30_2 = L30_2(L31_2)
    if L30_2 then
      L30_2 = L40_1
      L31_2 = L40_1
      L31_2 = #L31_2
      L31_2 = L31_2 + 1
      L30_2[L31_2] = L6_2
    end
    L30_2 = nil
    L31_2 = nil
    L32_2 = true
    return L30_2, L31_2, L32_2
  else
    L30_2 = pcall
    
    function L31_2()
      local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
      L0_3 = L24_2
      L1_3 = L0_3
      L0_3 = L0_3.read
      L2_3 = "*l"
      L0_3 = L0_3(L1_3, L2_3)
      L1_3 = assert
      L2_3 = L0_3 == "P6" or L0_3 == "P5"
      L3_3 = "Unsupported PPM format -> Expected binary RGB (P6) or Gray binary (P5). Got: "
      L4_3 = tostring
      L5_3 = L0_3
      L4_3 = L4_3(L5_3)
      L3_3 = L3_3 .. L4_3
      L1_3(L2_3, L3_3)
      L1_3 = L24_2
      L2_3 = L1_3
      L1_3 = L1_3.read
      L3_3 = "*l"
      L1_3 = L1_3(L2_3, L3_3)
      L2_3 = L24_2
      L3_3 = L2_3
      L2_3 = L2_3.read
      L4_3 = "*n"
      L2_3 = L2_3(L3_3, L4_3)
      L3_3 = L24_2
      L4_3 = L3_3
      L3_3 = L3_3.read
      L5_3 = "*n"
      L3_3 = L3_3(L4_3, L5_3)
      L4_3 = L24_2
      L5_3 = L4_3
      L4_3 = L4_3.read
      L6_3 = "*n"
      L4_3 = L4_3(L5_3, L6_3)
      L5_3 = assert
      L6_3 = L4_3 == 255
      L7_3 = "Unsupported bit depth. Expected 255, got: "
      L8_3 = tostring
      L9_3 = L4_3
      L8_3 = L8_3(L9_3)
      L7_3 = L7_3 .. L8_3
      L5_3(L6_3, L7_3)
      L5_3 = L24_2
      L6_3 = L5_3
      L5_3 = L5_3.read
      L7_3 = "*l"
      L5_3(L6_3, L7_3)
      L5_3 = L24_2
      L6_3 = L5_3
      L5_3 = L5_3.read
      L7_3 = "*a"
      L5_3 = L5_3(L6_3, L7_3)
      L6_3 = assert
      L7_3 = type
      L8_3 = L5_3
      L7_3 = L7_3(L8_3)
      L7_3 = L7_3 == "string"
      L8_3 = "Failed to read PPM pixel data or file is empty."
      L6_3(L7_3, L8_3)
      return L5_3
    end
    
    L30_2, L31_2 = L30_2(L31_2)
    L33_2 = L24_2
    L32_2 = L24_2.close
    L32_2(L33_2)
    if L30_2 and L31_2 then
      L29_2 = L31_2
    else
      L32_2 = L19_1
      L33_2 = L32_2
      L32_2 = L32_2.error
      L34_2 = "Failed to read PPM content for photo "
      L35_2 = A3_2
      L36_2 = ": "
      L37_2 = tostring
      L38_2 = L31_2
      L37_2 = L37_2(L38_2)
      L34_2 = L34_2 .. L35_2 .. L36_2 .. L37_2
      L32_2(L33_2, L34_2)
      L32_2 = L35_1
      L33_2 = L35_1
      L33_2 = #L33_2
      L33_2 = L33_2 + 1
      L34_2 = [[


Error reading content of PPM file for photo ]]
      L35_2 = A3_2
      L36_2 = ". File might be corrupted. "
      L37_2 = tostring
      L38_2 = L31_2
      L37_2 = L37_2(L38_2)
      L34_2 = L34_2 .. L35_2 .. L36_2 .. L37_2
      L32_2[L33_2] = L34_2
      L29_2 = nil
      L28_2 = nil
      L26_2 = true
      L32_2 = L7_1
      L32_2 = L32_2.exists
      L33_2 = L5_2
      L32_2 = L32_2(L33_2)
      if L32_2 then
        L32_2 = L40_1
        L33_2 = L40_1
        L33_2 = #L33_2
        L33_2 = L33_2 + 1
        L32_2[L33_2] = L5_2
      end
      L32_2 = L7_1
      L32_2 = L32_2.exists
      L33_2 = L6_2
      L32_2 = L32_2(L33_2)
      if L32_2 then
        L32_2 = L40_1
        L33_2 = L40_1
        L33_2 = #L33_2
        L33_2 = L33_2 + 1
        L32_2[L33_2] = L6_2
      end
      L32_2 = nil
      L33_2 = nil
      L34_2 = true
      return L32_2, L33_2, L34_2
    end
  end
  L30_2 = L29_2
  L31_2 = L28_2
  L32_2 = L26_2
  return L30_2, L31_2, L32_2
end

function L50_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2
  L4_2 = activePhotosMeta
  L4_2 = L4_2[A2_2]
  L4_2 = L4_2.settings
  L4_2 = L4_2.colorModel
  L5_2 = activePhotosMeta
  L5_2 = L5_2[A2_2]
  L5_2 = L5_2.settings
  L5_2 = L5_2.source
  L6_2 = activePhotosMeta
  L6_2 = L6_2[A2_2]
  L6_2 = L6_2.settings
  L6_2 = L6_2.imageType
  L7_2 = L39_1
  L7_2 = L7_2[L6_2]
  L7_2 = L7_2.white
  L8_2 = L39_1
  L8_2 = L8_2[L6_2]
  L8_2 = L8_2.black
  L9_2 = nil
  L10_2 = nil
  L11_2 = 0
  L12_2 = 1
  L13_2 = #A0_2
  L14_2 = 1
  for L15_2 = L12_2, L13_2, L14_2 do
    L16_2 = A0_2[L15_2]
    L16_2 = L16_2 / A1_2
    L11_2 = L11_2 + L16_2
    if L7_2 < L11_2 then
      L9_2 = L15_2 - 1
      if L9_2 < 0 then
        L9_2 = 0
      end
      break
    end
  end
  L11_2 = 0
  L12_2 = 256
  L13_2 = 1
  L14_2 = -1
  for L15_2 = L12_2, L13_2, L14_2 do
    L16_2 = A0_2[L15_2]
    L16_2 = L16_2 / A1_2
    L11_2 = L11_2 + L16_2
    if L8_2 < L11_2 then
      L10_2 = L15_2 - 1
      if 255 < L10_2 then
        L10_2 = 255
      end
      break
    end
  end
  L12_2 = L10_2 - L9_2
  L13_2 = 256 / L12_2
  L14_2 = 0.005
  L15_2 = 0.005
  L16_2 = L14_1
  L16_2 = L16_2.round
  L17_2 = 1 + L14_2
  L17_2 = L9_2 * L17_2
  L16_2 = L16_2(L17_2)
  L17_2 = L14_1
  L17_2 = L17_2.round
  L18_2 = 1 - L15_2
  L18_2 = L10_2 * L18_2
  L17_2 = L17_2(L18_2)
  L18_2 = L17_2 - L16_2
  L19_2 = 256 / L18_2
  L20_2 = 0
  L21_2 = 0
  L22_2 = L16_2 + 1
  L23_2 = L17_2 + 1
  L24_2 = 1
  for L25_2 = L22_2, L23_2, L24_2 do
    L26_2 = A0_2[L25_2]
    L21_2 = L21_2 + L26_2
    L26_2 = A0_2[L25_2]
    L27_2 = L25_2 - L16_2
    L27_2 = L27_2 - 1
    L27_2 = L27_2 * L19_2
    L28_2 = L26_2 * L27_2
    L20_2 = L20_2 + L28_2
  end
  L22_2 = L20_2 / L21_2
  L22_2 = L22_2 / 256
  L23_2 = A3_2.imageType
  if L23_2 ~= "positive" then
    L23_2 = A3_2.imageType
    if L23_2 ~= "editOnly" then
      L22_2 = 1 - L22_2
    end
  end
  L23_2 = L9_2
  L24_2 = L10_2
  L25_2 = L22_2
  return L23_2, L24_2, L25_2
end

function L51_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2
  L1_2 = {}
  L2_2 = {}
  L3_2 = {}
  L4_2 = 1
  L5_2 = 256
  L6_2 = 1
  for L7_2 = L4_2, L5_2, L6_2 do
    L1_2[L7_2] = 0
    L2_2[L7_2] = 0
    L3_2[L7_2] = 0
  end
  L4_2 = #A0_2
  L5_2 = L4_2
  L6_2 = 3
  L5_2 = L4_2 / L6_2
  L7_2 = 1
  L8_2 = L4_2 - L6_2
  L9_2 = L6_2
  for L10_2 = L7_2, L8_2, L9_2 do
    L12_2 = A0_2
    L11_2 = A0_2.byte
    L13_2 = L10_2
    L11_2 = L11_2(L12_2, L13_2)
    L11_2 = L11_2 + 1
    L13_2 = A0_2
    L12_2 = A0_2.byte
    L14_2 = L10_2 + 1
    L12_2 = L12_2(L13_2, L14_2)
    L12_2 = L12_2 + 1
    L14_2 = A0_2
    L13_2 = A0_2.byte
    L15_2 = L10_2 + 2
    L13_2 = L13_2(L14_2, L15_2)
    L13_2 = L13_2 + 1
    L14_2 = L1_2[L11_2]
    L14_2 = L14_2 + 1
    L1_2[L11_2] = L14_2
    L14_2 = L2_2[L12_2]
    L14_2 = L14_2 + 1
    L2_2[L12_2] = L14_2
    L14_2 = L3_2[L13_2]
    L14_2 = L14_2 + 1
    L3_2[L13_2] = L14_2
  end
  L7_2 = L1_2
  L8_2 = L2_2
  L9_2 = L3_2
  L10_2 = L5_2
  return L7_2, L8_2, L9_2, L10_2
end

L52_1 = false
L53_1 = 1

function L54_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2, A6_2, A7_2, A8_2)
  local L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2
  L9_2 = _G
  L9_2 = L9_2.activePhotosMeta
  L9_2 = L9_2[A3_2]
  L10_2 = 0
  L11_2 = A5_2 or L11_2
  L11_2 = A7_2 or L11_2
  L11_2 = A5_2 and A6_2 ~= nil and A7_2 and L11_2
  if not L9_2 then
    L12_2 = L19_1
    L13_2 = L12_2
    L12_2 = L12_2.error
    L14_2 = "Metadata container _G.activePhotosMeta["
    L15_2 = A3_2
    L16_2 = "] does not exist!"
    L14_2 = L14_2 .. L15_2 .. L16_2
    L12_2(L13_2, L14_2)
    L12_2 = L35_1
    L13_2 = L35_1
    L13_2 = #L13_2
    L13_2 = L13_2 + 1
    L14_2 = [[


Internal Error: Missing metadata container for photo ]]
    L15_2 = A3_2
    L14_2 = L14_2 .. L15_2
    L12_2[L13_2] = L14_2
    L12_2 = L10_2
    L13_2 = "failed"
    return L12_2, L13_2
  end
  L9_2.analysisStatus = "pending"
  L10_2 = L10_2 + 1
  if L11_2 then
    L12_2 = A8_2
    L13_2 = A6_2 + L10_2
    L14_2 = A7_2
    L12_2(L13_2, L14_2)
    L12_2 = L4_1
    L12_2 = L12_2.yield
    L12_2()
  end
  L12_2 = nil
  L13_2 = nil
  L14_2 = nil
  L15_2 = L49_1
  if L15_2 then
    L15_2 = L49_1
    L16_2 = A0_2
    L17_2 = A1_2
    L18_2 = A2_2
    L19_2 = A3_2
    L20_2 = A4_2
    L15_2, L16_2, L17_2 = L15_2(L16_2, L17_2, L18_2, L19_2, L20_2)
    L14_2 = L17_2
    L13_2 = L16_2
    L12_2 = L15_2
  else
    L15_2 = L19_1
    L16_2 = L15_2
    L15_2 = L15_2.error
    L17_2 = "decodeJpeg function missing!"
    L15_2(L16_2, L17_2)
    L9_2.analysisStatus = "failed"
    L15_2 = L35_1
    L16_2 = L35_1
    L16_2 = #L16_2
    L16_2 = L16_2 + 1
    L17_2 = [[


Internal Error: decodeJpeg function missing for photo ]]
    L18_2 = A3_2
    L17_2 = L17_2 .. L18_2
    L15_2[L16_2] = L17_2
    L15_2 = L9_2.redChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.redChannel = L15_2
    L15_2 = L9_2.greenChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.greenChannel = L15_2
    L15_2 = L9_2.blueChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.blueChannel = L15_2
    L15_2 = L9_2.redChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.redChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.redChannel
    L15_2.meanPoint = nil
    L15_2 = L9_2.greenChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.greenChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.greenChannel
    L15_2.meanPoint = nil
    L15_2 = L9_2.blueChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.blueChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.blueChannel
    L15_2.meanPoint = nil
    L9_2.autoWB = nil
    L9_2.originsComputed = false
    L15_2 = _G
    L15_2 = L15_2.activePhotoIndex
    if L15_2 == A3_2 then
      L15_2 = _G
      L15_2.isConverted = false
    end
    L15_2 = L10_2
    L16_2 = "failed"
    return L15_2, L16_2
  end
  if L12_2 == nil then
    L15_2 = L19_1
    L16_2 = L15_2
    L15_2 = L15_2.error
    L17_2 = string
    L17_2 = L17_2.format
    L18_2 = "decodeJpeg returned nil for photo %d. Analysis failed."
    L19_2 = A3_2
    L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L17_2(L18_2, L19_2)
    L15_2(L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2)
    L9_2.analysisStatus = "failed"
    L15_2 = L9_2.redChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.redChannel = L15_2
    L15_2 = L9_2.greenChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.greenChannel = L15_2
    L15_2 = L9_2.blueChannel
    if not L15_2 then
      L15_2 = {}
    end
    L9_2.blueChannel = L15_2
    L15_2 = L9_2.redChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.redChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.redChannel
    L15_2.meanPoint = nil
    L15_2 = L9_2.greenChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.greenChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.greenChannel
    L15_2.meanPoint = nil
    L15_2 = L9_2.blueChannel
    L15_2.whitePointOrigin = nil
    L15_2 = L9_2.blueChannel
    L15_2.blackPointOrigin = nil
    L15_2 = L9_2.blueChannel
    L15_2.meanPoint = nil
    L9_2.autoWB = nil
    L9_2.originsComputed = false
    L15_2 = _G
    L15_2 = L15_2.activePhotoIndex
    if L15_2 == A3_2 then
      L15_2 = _G
      L15_2.isConverted = false
    end
    L15_2 = L10_2
    L16_2 = "failed"
    return L15_2, L16_2
  end
  L10_2 = L10_2 + 1
  if L11_2 then
    L15_2 = A8_2
    L16_2 = A6_2 + L10_2
    L17_2 = A7_2
    L15_2(L16_2, L17_2)
    L15_2 = L4_1
    L15_2 = L15_2.yield
    L15_2()
  end
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = nil
  L19_2 = L51_1
  if L19_2 then
    L19_2 = L51_1
    L20_2 = L12_2
    L19_2, L20_2, L21_2, L22_2 = L19_2(L20_2)
    L18_2 = L22_2
    L17_2 = L21_2
    L16_2 = L20_2
    L15_2 = L19_2
    L12_2 = nil
  else
    L19_2 = L19_1
    L20_2 = L19_2
    L19_2 = L19_2.error
    L21_2 = "computeHistograms function missing!"
    L19_2(L20_2, L21_2)
    L9_2.analysisStatus = "failed"
    L19_2 = L35_1
    L20_2 = L35_1
    L20_2 = #L20_2
    L20_2 = L20_2 + 1
    L21_2 = [[


Internal Error: computeHistograms function missing for photo ]]
    L22_2 = A3_2
    L21_2 = L21_2 .. L22_2
    L19_2[L20_2] = L21_2
    L19_2 = L9_2.redChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.redChannel = L19_2
    L19_2 = L9_2.greenChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.greenChannel = L19_2
    L19_2 = L9_2.blueChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.blueChannel = L19_2
    L19_2 = L9_2.redChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.redChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.redChannel
    L19_2.meanPoint = nil
    L19_2 = L9_2.greenChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.greenChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.greenChannel
    L19_2.meanPoint = nil
    L19_2 = L9_2.blueChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.blueChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.blueChannel
    L19_2.meanPoint = nil
    L9_2.autoWB = nil
    L9_2.originsComputed = false
    L19_2 = _G
    L19_2 = L19_2.activePhotoIndex
    if L19_2 == A3_2 then
      L19_2 = _G
      L19_2.isConverted = false
    end
    L19_2 = L10_2
    L20_2 = "failed"
    return L19_2, L20_2
  end
  if not (L15_2 and L16_2 and L17_2 and L18_2) or L18_2 == 0 then
    L19_2 = L19_1
    L20_2 = L19_2
    L19_2 = L19_2.error
    L21_2 = "computeHistograms returned invalid data for photo "
    L22_2 = A3_2
    L21_2 = L21_2 .. L22_2
    L19_2(L20_2, L21_2)
    L19_2 = L35_1
    L20_2 = L35_1
    L20_2 = #L20_2
    L20_2 = L20_2 + 1
    L21_2 = [[


Error computing histogram data for photo ]]
    L22_2 = A3_2
    L21_2 = L21_2 .. L22_2
    L19_2[L20_2] = L21_2
    L9_2.analysisStatus = "failed"
    L19_2 = L9_2.redChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.redChannel = L19_2
    L19_2 = L9_2.greenChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.greenChannel = L19_2
    L19_2 = L9_2.blueChannel
    if not L19_2 then
      L19_2 = {}
    end
    L9_2.blueChannel = L19_2
    L19_2 = L9_2.redChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.redChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.redChannel
    L19_2.meanPoint = nil
    L19_2 = L9_2.greenChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.greenChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.greenChannel
    L19_2.meanPoint = nil
    L19_2 = L9_2.blueChannel
    L19_2.whitePointOrigin = nil
    L19_2 = L9_2.blueChannel
    L19_2.blackPointOrigin = nil
    L19_2 = L9_2.blueChannel
    L19_2.meanPoint = nil
    L9_2.autoWB = nil
    L9_2.originsComputed = false
    L19_2 = _G
    L19_2 = L19_2.activePhotoIndex
    if L19_2 == A3_2 then
      L19_2 = _G
      L19_2.isConverted = false
    end
    L19_2 = L10_2
    L20_2 = "failed"
    return L19_2, L20_2
  end
  L10_2 = L10_2 + 1
  if L11_2 then
    L19_2 = A8_2
    L20_2 = A6_2 + L10_2
    L21_2 = A7_2
    L19_2(L20_2, L21_2)
    L19_2 = L4_1
    L19_2 = L19_2.yield
    L19_2()
  end
  L19_2 = nil
  L20_2 = nil
  L21_2 = nil
  L22_2 = nil
  L23_2 = nil
  L24_2 = nil
  L25_2 = nil
  L26_2 = nil
  L27_2 = nil
  L28_2 = L50_1
  if L28_2 then
    L28_2 = L50_1
    L29_2 = L15_2
    L30_2 = L18_2
    L31_2 = A3_2
    L32_2 = A2_2
    L33_2 = "red"
    L28_2, L29_2, L30_2 = L28_2(L29_2, L30_2, L31_2, L32_2, L33_2)
    L21_2 = L30_2
    L20_2 = L29_2
    L19_2 = L28_2
    L28_2 = L50_1
    L29_2 = L16_2
    L30_2 = L18_2
    L31_2 = A3_2
    L32_2 = A2_2
    L33_2 = "green"
    L28_2, L29_2, L30_2 = L28_2(L29_2, L30_2, L31_2, L32_2, L33_2)
    L24_2 = L30_2
    L23_2 = L29_2
    L22_2 = L28_2
    L28_2 = L50_1
    L29_2 = L17_2
    L30_2 = L18_2
    L31_2 = A3_2
    L32_2 = A2_2
    L33_2 = "blue"
    L28_2, L29_2, L30_2 = L28_2(L29_2, L30_2, L31_2, L32_2, L33_2)
    L27_2 = L30_2
    L26_2 = L29_2
    L25_2 = L28_2
  else
    L28_2 = L19_1
    L29_2 = L28_2
    L28_2 = L28_2.error
    L30_2 = "computeBaseCurve function missing!"
    L28_2(L29_2, L30_2)
    L9_2.analysisStatus = "failed"
    L28_2 = L35_1
    L29_2 = L35_1
    L29_2 = #L29_2
    L29_2 = L29_2 + 1
    L30_2 = [[


Internal Error: computeBaseCurve function missing for photo ]]
    L31_2 = A3_2
    L30_2 = L30_2 .. L31_2
    L28_2[L29_2] = L30_2
    L28_2 = L9_2.redChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.redChannel = L28_2
    L28_2 = L9_2.greenChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.greenChannel = L28_2
    L28_2 = L9_2.blueChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.blueChannel = L28_2
    L28_2 = L9_2.redChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.redChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.redChannel
    L28_2.meanPoint = nil
    L28_2 = L9_2.greenChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.greenChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.greenChannel
    L28_2.meanPoint = nil
    L28_2 = L9_2.blueChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.blueChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.blueChannel
    L28_2.meanPoint = nil
    L9_2.autoWB = L13_2
    L9_2.originsComputed = false
    L28_2 = _G
    L28_2 = L28_2.activePhotoIndex
    if L28_2 == A3_2 then
      L28_2 = _G
      L28_2.isConverted = false
    end
    L28_2 = L10_2
    L29_2 = "failed"
    return L28_2, L29_2
  end
  if not (L19_2 and L20_2 and L21_2 and L21_2 == L21_2 and L22_2 and L23_2 and L24_2 and L24_2 == L24_2 and L25_2 and L26_2 and L27_2) or L27_2 ~= L27_2 then
    L28_2 = L19_1
    L29_2 = L28_2
    L28_2 = L28_2.error
    L30_2 = "computeBaseCurve returned invalid data for photo "
    L31_2 = A3_2
    L30_2 = L30_2 .. L31_2
    L28_2(L29_2, L30_2)
    L28_2 = L35_1
    L29_2 = L35_1
    L29_2 = #L29_2
    L29_2 = L29_2 + 1
    L30_2 = [[


Error calculating image analysis data for photo ]]
    L31_2 = A3_2
    L30_2 = L30_2 .. L31_2
    L28_2[L29_2] = L30_2
    L9_2.analysisStatus = "failed"
    L28_2 = L9_2.redChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.redChannel = L28_2
    L28_2 = L9_2.greenChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.greenChannel = L28_2
    L28_2 = L9_2.blueChannel
    if not L28_2 then
      L28_2 = {}
    end
    L9_2.blueChannel = L28_2
    L28_2 = L9_2.redChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.redChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.redChannel
    L28_2.meanPoint = nil
    L28_2 = L9_2.greenChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.greenChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.greenChannel
    L28_2.meanPoint = nil
    L28_2 = L9_2.blueChannel
    L28_2.whitePointOrigin = nil
    L28_2 = L9_2.blueChannel
    L28_2.blackPointOrigin = nil
    L28_2 = L9_2.blueChannel
    L28_2.meanPoint = nil
    L9_2.autoWB = L13_2
    L9_2.originsComputed = false
    L28_2 = _G
    L28_2 = L28_2.activePhotoIndex
    if L28_2 == A3_2 then
      L28_2 = _G
      L28_2.isConverted = false
    end
    L28_2 = L10_2
    L29_2 = "failed"
    return L28_2, L29_2
  end
  L28_2 = L9_2.redChannel
  if not L28_2 then
    L28_2 = {}
  end
  L9_2.redChannel = L28_2
  L28_2 = L9_2.greenChannel
  if not L28_2 then
    L28_2 = {}
  end
  L9_2.greenChannel = L28_2
  L28_2 = L9_2.blueChannel
  if not L28_2 then
    L28_2 = {}
  end
  L9_2.blueChannel = L28_2
  L28_2 = L9_2.redChannel
  L28_2.whitePointOrigin = L19_2
  L28_2 = L9_2.redChannel
  L28_2.blackPointOrigin = L20_2
  L28_2 = L9_2.redChannel
  L28_2.meanPoint = L21_2
  L28_2 = L9_2.greenChannel
  L28_2.whitePointOrigin = L22_2
  L28_2 = L9_2.greenChannel
  L28_2.blackPointOrigin = L23_2
  L28_2 = L9_2.greenChannel
  L28_2.meanPoint = L24_2
  L28_2 = L9_2.blueChannel
  L28_2.whitePointOrigin = L25_2
  L28_2 = L9_2.blueChannel
  L28_2.blackPointOrigin = L26_2
  L28_2 = L9_2.blueChannel
  L28_2.meanPoint = L27_2
  L9_2.originsComputed = true
  if L13_2 then
    L9_2.autoWB = L13_2
    if L14_2 then
      L28_2 = "partial"
      if L28_2 then
        goto lbl_550
      end
    end
    L28_2 = "success"
    ::lbl_550::
    L9_2.analysisStatus = L28_2
  else
    L28_2 = L19_1
    L29_2 = L28_2
    L28_2 = L28_2.error
    L30_2 = "autoWB is nil after successful analysis steps for photo "
    L31_2 = A3_2
    L30_2 = L30_2 .. L31_2
    L28_2(L29_2, L30_2)
    L9_2.analysisStatus = "failed"
    L9_2.autoWB = nil
    L28_2 = _G
    L28_2 = L28_2.activePhotoIndex
    if L28_2 == A3_2 then
      L28_2 = _G
      L28_2.isConverted = false
    end
  end
  L28_2 = L9_2.analysisStatus
  if L28_2 ~= "success" then
    L28_2 = L9_2.analysisStatus
    if L28_2 ~= "partial" then
      goto lbl_583
    end
  end
  L10_2 = L10_2 + 1
  if L11_2 then
    L28_2 = A8_2
    L29_2 = A6_2 + L10_2
    L30_2 = A7_2
    L28_2(L29_2, L30_2)
    L28_2 = L4_1
    L28_2 = L28_2.yield
    L28_2()
    goto lbl_589
    ::lbl_583::
    L28_2 = L19_1
    L29_2 = L28_2
    L28_2 = L28_2.warn
    L30_2 = "Analysis finalization failed (likely nil autoWB) for photo "
    L31_2 = A3_2
    L30_2 = L30_2 .. L31_2
    L28_2(L29_2, L30_2)
  end
  ::lbl_589::
  L28_2 = _G
  L28_2 = L28_2.activePhotoIndex
  if L28_2 == A3_2 then
    L28_2 = _G
    L29_2 = L9_2.analysisStatus
    L29_2 = L29_2 == "success"
    L28_2.isConverted = L29_2
  end
  L28_2 = L10_2
  L29_2 = L9_2.analysisStatus
  return L28_2, L29_2
end

function L55_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L0_2 = L3_1
  L0_2 = L0_2.osFactory
  L0_2 = L0_2()
  
  function L1_2(A0_3, A1_3)
    local L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
    L2_3 = 0
    L4_3 = A0_3
    L3_3 = A0_3.gmatch
    L5_3 = "[^\r\n]+"
    L3_3, L4_3, L5_3 = L3_3(L4_3, L5_3)
    for L6_3 in L3_3, L4_3, L5_3 do
      L7_3 = #L6_3
      L8_3 = math
      L8_3 = L8_3.ceil
      L9_3 = L7_3 / A1_3
      L8_3 = L8_3(L9_3)
      L2_3 = L2_3 + L8_3
    end
    L3_3 = math
    L3_3 = L3_3.max
    L4_3 = L2_3
    L5_3 = 10
    L3_3 = L3_3(L4_3, L5_3)
    return L3_3
  end
  
  L2_2 = table
  L2_2 = L2_2.concat
  L3_2 = L35_1
  L4_2 = "\n"
  L2_2 = L2_2(L3_2, L4_2)
  L3_2 = 50
  L4_2 = L1_2
  L5_2 = L2_2
  L6_2 = L3_2
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = L35_1
  L5_2 = #L5_2
  if 1 < L5_2 then
    L5_2 = "errors"
    if L5_2 then
      goto lbl_23
    end
  end
  L5_2 = "error"
  ::lbl_23::
  L7_2 = L0_2
  L6_2 = L0_2.column
  L8_2 = {}
  L10_2 = L0_2
  L9_2 = L0_2.control_spacing
  L9_2 = L9_2(L10_2)
  L8_2.spacing = L9_2
  L10_2 = L0_2
  L9_2 = L0_2.static_text
  L11_2 = {}
  L12_2 = "WARNING: "
  L13_2 = L35_1
  L13_2 = #L13_2
  L14_2 = " "
  L15_2 = L5_2
  L16_2 = " occurred during analysis."
  L12_2 = L12_2 .. L13_2 .. L14_2 .. L15_2 .. L16_2
  L11_2.title = L12_2
  L11_2.fill_horizontal = 1
  L11_2.font = "<system/bold>"
  L12_2 = L12_1
  L13_2 = 1
  L14_2 = 0
  L15_2 = 0
  L12_2 = L12_2(L13_2, L14_2, L15_2)
  L11_2.text_color = L12_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = L0_2
  L10_2 = L0_2.scrolled_view
  L12_2 = {}
  L12_2.width = 600
  L12_2.height = 300
  L12_2.horizontal_scroller = false
  L14_2 = L0_2
  L13_2 = L0_2.edit_field
  L15_2 = {}
  L15_2.value = L2_2
  L15_2.width_in_chars = L3_2
  L15_2.height_in_lines = L4_2
  L15_2.multiline = true
  L15_2.enabled = true
  L13_2, L14_2, L15_2, L16_2 = L13_2(L14_2, L15_2)
  L12_2[1] = L13_2
  L12_2[2] = L14_2
  L12_2[3] = L15_2
  L12_2[4] = L16_2
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2 = L10_2(L11_2, L12_2)
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L8_2[4] = L12_2
  L8_2[5] = L13_2
  L8_2[6] = L14_2
  L8_2[7] = L15_2
  L8_2[8] = L16_2
  L6_2 = L6_2(L7_2, L8_2)
  L7_2 = L2_1
  L7_2 = L7_2.presentModalDialog
  L8_2 = {}
  L8_2.title = "Errors"
  L8_2.contents = L6_2
  L8_2.actionVerb = "Continue"
  L8_2.cancelVerb = nil
  L7_2 = L7_2(L8_2)
  L8_2 = L7_2
  L9_2 = L2_2
  return L8_2, L9_2
end

function L56_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = 1
  L3_2 = #A0_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.redChannel
    L6_2.whitePointOrigin = 0
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.redChannel
    L6_2.blackPointOrigin = 255
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.redChannel
    L6_2.meanPoint = 0.5
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.greenChannel
    L6_2.whitePointOrigin = 0
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.greenChannel
    L6_2.blackPointOrigin = 255
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.greenChannel
    L6_2.meanPoint = 0.5
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.blueChannel
    L6_2.whitePointOrigin = 0
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.blueChannel
    L6_2.blackPointOrigin = 255
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2 = L6_2.blueChannel
    L6_2.meanPoint = 0.5
    L6_2 = activePhotosMeta
    L6_2 = L6_2[L5_2]
    L7_2 = {}
    L8_2 = {}
    L8_2.h = 0
    L8_2.s = 0
    L8_2.b = 0.5
    L7_2.smartColorNeutral = L8_2
    L8_2 = {}
    L8_2.h = 0
    L8_2.s = 0
    L8_2.b = 0.5
    L7_2.smartColorWarming = L8_2
    L8_2 = {}
    L8_2.h = 0
    L8_2.s = 0
    L8_2.b = 0.5
    L7_2.smartColorCooling = L8_2
    L6_2.autoWB = L7_2
  end
  L2_2 = NLPDialogs
  L2_2 = L2_2.mainDialog
  L2_2()
end

L57_1 = NLPImageProcessor

function L58_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L3_2 = A1_2.imageType
  if L3_2 == "editOnly" then
    L3_2 = L56_1
    L4_2 = A0_2
    L5_2 = A1_2
    L3_2(L4_2, L5_2)
    return
  end
  L3_2 = L9_1
  L3_2 = L3_2.stopTracking
  L3_2()
  L3_2 = L1_1
  L3_2 = L3_2.callWithContext
  L4_2 = "export"
  
  function L5_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3, L16_3, L17_3, L18_3, L19_3, L20_3
    L1_3 = L2_1
    L1_3 = L1_3.showModalProgressDialog
    L2_3 = {}
    L2_3.title = "Processing Film..."
    L2_3.caption = "Loading..."
    L2_3.cannotCancel = false
    L2_3.functionContext = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = {}
    L2_3.LR_export_destinationType = "specificFolder"
    L3_3 = L21_1
    L2_3.LR_export_destinationPathPrefix = L3_3
    L2_3.LR_export_useSubfolder = false
    L2_3.LR_reimportExportedPhoto = false
    L2_3.LR_collisionHandling = "rename"
    L2_3.LR_renamingTokensOn = true
    L2_3.LR_tokens = "{{naming_sequenceTotal_5Digits}}"
    L2_3.LR_format = "TIFF"
    L2_3.LRtiff_compressionMethod = "compressionMethod_None"
    L2_3.LR_jpeg_quality = 1
    L2_3.LR_export_colorSpace = "ProPhotoRGB"
    L2_3.LR_export_bitDepth = 8
    L2_3.LR_size_resizeType = "longEdge"
    L2_3.LR_size_maxHeight = 200
    L2_3.LR_size_doConstrain = true
    L2_3.LR_size_doNotEnlarge = true
    L2_3.LR_size_units = "pixels"
    L2_3.LR_useWatermark = false
    L2_3.LR_outputSharpeningOn = false
    L2_3.LR_minimizeEmbeddedMetadata = true
    L3_3 = L10_1
    L4_3 = {}
    L5_3 = A0_2
    L4_3.photosToExport = L5_3
    L4_3.exportSettings = L2_3
    L3_3 = L3_3(L4_3)
    L5_3 = L3_3
    L4_3 = L3_3.countRenditions
    L4_3 = L4_3(L5_3)
    L5_3 = {}
    L5_3.progressScope = L1_3
    L5_3.renderProgressPortion = 1
    L5_3.stopIfCanceled = true
    L7_3 = L3_3
    L6_3 = L3_3.renditions
    L8_3 = L5_3
    L6_3, L7_3, L8_3 = L6_3(L7_3, L8_3)
    for L9_3, L10_3 in L6_3, L7_3, L8_3 do
      L11_3 = L10_3.photo
      L12_3 = L11_3
      L11_3 = L11_3.getFormattedMetadata
      L13_3 = "fileName"
      L11_3 = L11_3(L12_3, L13_3)
      L13_3 = L1_3
      L12_3 = L1_3.setCaption
      L14_3 = "Rendering "
      L15_3 = L11_3
      L16_3 = " ("
      L17_3 = L9_3
      L18_3 = " of "
      L19_3 = L4_3
      L20_3 = ")"
      L14_3 = L14_3 .. L15_3 .. L16_3 .. L17_3 .. L18_3 .. L19_3 .. L20_3
      L12_3(L13_3, L14_3)
      L13_3 = L1_3
      L12_3 = L1_3.setPortionComplete
      L14_3 = L9_3 - 1
      L14_3 = L14_3 + 0.33
      L15_3 = L4_3
      L12_3(L13_3, L14_3, L15_3)
      L13_3 = L10_3
      L12_3 = L10_3.waitForRender
      L12_3(L13_3)
      L13_3 = L1_3
      L12_3 = L1_3.isCanceled
      L12_3 = L12_3(L13_3)
      if L12_3 then
        break
      end
      L13_3 = L1_3
      L12_3 = L1_3.setCaption
      L14_3 = "Analyzing "
      L15_3 = L11_3
      L16_3 = " ("
      L17_3 = L9_3
      L18_3 = " of "
      L19_3 = L4_3
      L20_3 = ")"
      L14_3 = L14_3 .. L15_3 .. L16_3 .. L17_3 .. L18_3 .. L19_3 .. L20_3
      L12_3(L13_3, L14_3)
      L13_3 = L1_3
      L12_3 = L1_3.setPortionComplete
      L14_3 = L9_3 - 1
      L14_3 = L14_3 + 0.66
      L15_3 = L4_3
      L12_3(L13_3, L14_3, L15_3)
      L12_3 = L1_1
      L12_3 = L12_3.callWithContext
      L13_3 = "decode and compute photos"
      
      function L14_3(A0_4)
        local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4
        L1_4 = L10_3
        L1_4 = L1_4.destinationPath
        L2_4 = L10_3
        L2_4 = L2_4.photo
        L3_4 = L2_4
        L2_4 = L2_4.getRawMetadata
        L4_4 = "uuid"
        L2_4 = L2_4(L3_4, L4_4)
        L3_4 = assert
        L4_4 = L7_1
        L4_4 = L4_4.isReadable
        L5_4 = L1_4
        L4_4 = L4_4(L5_4)
        L5_4 = [[


ERROR: Could not read photo for analysis.

Path: ]]
        L6_4 = L1_4
        L7_4 = [[


Make sure the photo is available. If the photo is on an external drive, make sure that drive is connected.]]
        L5_4 = L5_4 .. L6_4 .. L7_4
        L3_4(L4_4, L5_4)
        L3_4 = queueForAnalysis
        L4_4 = L1_4
        L5_4 = L2_4
        L6_4 = A1_2
        L7_4 = L1_3
        L8_4 = L9_3
        L9_4 = L4_3
        L3_4(L4_4, L5_4, L6_4, L7_4, L8_4, L9_4)
        L3_4 = L19_1
        L4_4 = L3_4
        L3_4 = L3_4.trace
        L5_4 = "Finished analysis on photo "
        L6_4 = L9_3
        L5_4 = L5_4 .. L6_4
        L3_4(L4_4, L5_4)
      end
      
      L12_3(L13_3, L14_3)
    end
    L7_3 = L1_3
    L6_3 = L1_3.isCanceled
    L6_3 = L6_3(L7_3)
    if L6_3 then
      return
    end
  end
  
  L3_2(L4_2, L5_2)
  L3_2 = A1_2.convertAsRoll
  if L3_2 == true then
    L3_2 = #A0_2
    if 1 < L3_2 then
      L3_2 = _G
      L3_2 = L3_2.activePhoto
      L4_2 = L3_2
      L3_2 = L3_2.getRawMetadata
      L5_2 = "path"
      L3_2 = L3_2(L4_2, L5_2)
      L4_2 = L6_1
      L4_2 = L4_2.parent
      L5_2 = L3_2
      L4_2 = L4_2(L5_2)
      L5_2 = L6_1
      L5_2 = L5_2.leafName
      L6_2 = L4_2
      L5_2 = L5_2(L6_2)
      L6_2 = A0_2
      L7_2 = L5_2
      L8_2 = " ("
      L9_2 = #A0_2
      L10_2 = ")"
      L7_2 = L7_2 .. L8_2 .. L9_2 .. L10_2
      L8_2 = "User"
      L9_2 = true
      L10_2 = true
      L11_2 = L18_1
      L11_2 = L11_2.createRoll
      L12_2 = L6_2
      L13_2 = L7_2
      L14_2 = L8_2
      L15_2 = L9_2
      L16_2 = L10_2
      L11_2 = L11_2(L12_2, L13_2, L14_2, L15_2, L16_2)
      L12_2 = 1
      L13_2 = #A0_2
      L14_2 = 1
      for L15_2 = L12_2, L13_2, L14_2 do
        L16_2 = _G
        L16_2 = L16_2.activePhotosMeta
        L16_2 = L16_2[L15_2]
        L16_2 = L16_2.settings
        L16_2.rollAnalysis = L11_2
      end
    end
  end
  L3_2 = L11_1
  L4_2 = {}
  L5_2 = A1_2.source
  L4_2.source = L5_2
  L5_2 = A1_2.filmBorder
  L4_2.filmBorder = L5_2
  L5_2 = A1_2.saturation
  L4_2.saturation = L5_2
  L5_2 = A1_2.colorModel
  L4_2.colorModel = L5_2
  L5_2 = A1_2.previewBuffer
  L4_2.previewBuffer = L5_2
  L5_2 = A1_2.convertAsRoll
  L4_2.convertAsRoll = L5_2
  L5_2 = A1_2.imageType
  L4_2.imageType = L5_2
  L3_2.conversionSettings = L4_2
  L3_2 = L11_1
  L4_2 = L11_1
  L4_2 = L4_2.conversionSettings
  L3_2.conversionSettings = L4_2
  L3_2 = NLPDialogs
  L3_2 = L3_2.mainDialog
  L3_2()
end

L57_1.processPhoto = L58_1
L57_1 = NLPImageProcessor

function L58_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L3_2 = {}
  L35_1 = L3_2
  L3_2 = {}
  L40_1 = L3_2
  L3_2 = A1_2.imageType
  if L3_2 == "editOnly" then
    L3_2 = L56_1
    L4_2 = A0_2
    L5_2 = A1_2
    L3_2(L4_2, L5_2)
    return
  end
  L3_2 = pcall
  L4_2 = L9_1
  L4_2 = L4_2.stopTracking
  L3_2(L4_2)
  L3_2 = true
  L4_2 = {}
  L5_2 = {}
  L6_2 = L1_1
  L6_2 = L6_2.callWithContext
  L7_2 = "ProcessBatch"
  
  function L8_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3, L16_3, L17_3, L18_3, L19_3, L20_3, L21_3, L22_3, L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3
    L2_3 = A0_3
    L1_3 = A0_3.addCleanupHandler
    
    function L3_3()
      local L0_4, L1_4
      L0_4 = L43_1
      L0_4()
      L0_4 = _G
      L0_4.testScenario = nil
      L0_4 = _G
      L0_4.testFailPhotoIndex = nil
    end
    
    L1_3(L2_3, L3_3)
    L1_3 = L2_1
    L1_3 = L1_3.showModalProgressDialog
    L2_3 = {}
    L2_3.title = "Processing Film..."
    L2_3.caption = "Initializing..."
    L2_3.cannotCancel = true
    L2_3.functionContext = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = {}
    L2_3.LR_export_destinationType = "specificFolder"
    L3_3 = L21_1
    L2_3.LR_export_destinationPathPrefix = L3_3
    L2_3.LR_export_useSubfolder = false
    L2_3.LR_reimportExportedPhoto = false
    L2_3.LR_collisionHandling = "rename"
    L2_3.LR_renamingTokensOn = true
    L2_3.LR_tokens = "{{naming_sequenceTotal_5Digits}}"
    L2_3.LR_format = "TIFF"
    L2_3.LRtiff_compressionMethod = "compressionMethod_None"
    L2_3.LR_export_colorSpace = "ProPhotoRGB"
    L2_3.LR_export_bitDepth = 8
    L2_3.LR_size_resizeType = "longEdge"
    L2_3.LR_size_maxHeight = 600
    L2_3.LR_size_doConstrain = true
    L2_3.LR_size_doNotEnlarge = true
    L2_3.LR_size_units = "pixels"
    L2_3.LR_useWatermark = false
    L2_3.LR_outputSharpeningOn = false
    L2_3.LR_minimizeEmbeddedMetadata = true
    L3_3 = L10_1
    L4_3 = {}
    L5_3 = A0_2
    L4_3.photosToExport = L5_3
    L4_3.exportSettings = L2_3
    L3_3 = L3_3(L4_3)
    L5_3 = L3_3
    L4_3 = L3_3.countRenditions
    L4_3 = L4_3(L5_3)
    if L4_3 == 0 then
      L5_3 = L19_1
      L6_3 = L5_3
      L5_3 = L5_3.warn
      L7_3 = "No photos to process."
      L5_3(L6_3, L7_3)
      L6_3 = L1_3
      L5_3 = L1_3.close
      L5_3(L6_3)
      return
    end
    L5_3 = 6
    L6_3 = L4_3 * L5_3
    L7_3 = 0
    
    function L8_3(A0_4, A1_4)
      local L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4
      L2_4 = L1_3
      L3_4 = L2_4
      L2_4 = L2_4.isCanceled
      L2_4 = L2_4(L3_4)
      if L2_4 then
        return
      end
      L2_4 = 0
      if 0 < A1_4 then
        L2_4 = A0_4 / A1_4
      end
      L3_4 = math
      L3_4 = L3_4.max
      L4_4 = 0
      L5_4 = math
      L5_4 = L5_4.min
      L6_4 = 1
      L7_4 = L2_4
      L5_4, L6_4, L7_4, L8_4, L9_4 = L5_4(L6_4, L7_4)
      L3_4 = L3_4(L4_4, L5_4, L6_4, L7_4, L8_4, L9_4)
      L2_4 = L3_4
      L3_4 = math
      L3_4 = L3_4.floor
      L4_4 = L2_4 * 100
      L3_4 = L3_4(L4_4)
      L4_4 = ""
      L5_4 = A1_2
      L5_4 = L5_4.convertAsRoll
      if L5_4 then
        if L3_4 < 15 then
          L4_4 = "Preparing photos..."
        elseif L3_4 < 35 then
          L4_4 = "Beginning analysis..."
        elseif L3_4 < 55 then
          L4_4 = "Calculating film dynamics..."
        elseif L3_4 < 75 then
          L4_4 = "Correcting color shifts..."
        elseif L3_4 < 95 then
          L4_4 = "Evaluating roll profile..."
        else
          L4_4 = "Finishing..."
        end
      elseif L3_4 < 25 then
        L4_4 = "Preparing photos..."
      elseif L3_4 < 40 then
        L4_4 = "Beginning analysis..."
      elseif L3_4 < 65 then
        L4_4 = "Calculating film dynamics..."
      elseif L3_4 < 95 then
        L4_4 = "Correcting color shifts..."
      else
        L4_4 = "Finishing up..."
      end
      L5_4 = string
      L5_4 = L5_4.format
      L6_4 = "%s (%d%% Complete)"
      L7_4 = L4_4
      L8_4 = L3_4
      L5_4 = L5_4(L6_4, L7_4, L8_4)
      L6_4 = L1_3
      L7_4 = L6_4
      L6_4 = L6_4.setCaption
      L8_4 = L5_4
      L6_4(L7_4, L8_4)
      L6_4 = L1_3
      L7_4 = L6_4
      L6_4 = L6_4.setPortionComplete
      L8_4 = L2_4
      L9_4 = 1
      L6_4(L7_4, L8_4, L9_4)
    end
    
    L9_3 = {}
    L9_3.stopIfCanceled = false
    L10_3 = L8_3
    L11_3 = L7_3
    L12_3 = L6_3
    L10_3(L11_3, L12_3)
    L10_3 = L4_1
    L10_3 = L10_3.yield
    L10_3()
    L11_3 = L3_3
    L10_3 = L3_3.renditions
    L12_3 = L9_3
    L10_3, L11_3, L12_3 = L10_3(L11_3, L12_3)
    for L13_3, L14_3 in L10_3, L11_3, L12_3 do
      L16_3 = L1_3
      L15_3 = L1_3.isCanceled
      L15_3 = L15_3(L16_3)
      if L15_3 then
        L15_3 = L19_1
        L16_3 = L15_3
        L15_3 = L15_3.warn
        L17_3 = "User cancelled operation during export loop start."
        L15_3(L16_3, L17_3)
        L15_3 = false
        L3_2 = L15_3
        break
      end
      L15_3 = L14_3.photo
      L17_3 = L15_3
      L16_3 = L15_3.getFormattedMetadata
      L18_3 = "fileName"
      L16_3 = L16_3(L17_3, L18_3)
      if not L16_3 then
        L16_3 = "photo_"
        L17_3 = L13_3
        L16_3 = L16_3 .. L17_3
      end
      L18_3 = L15_3
      L17_3 = L15_3.getRawMetadata
      L19_3 = "uuid"
      L17_3 = L17_3(L18_3, L19_3)
      if not L17_3 then
        L17_3 = "uuid_"
        L18_3 = L13_3
        L19_3 = "_"
        L20_3 = L14_1
        L20_3 = L20_3.generateUUID
        L20_3 = L20_3()
        L17_3 = L17_3 .. L18_3 .. L19_3 .. L20_3
      end
      L7_3 = L7_3 + 1
      L18_3 = L8_3
      L19_3 = L7_3
      L20_3 = L6_3
      L18_3(L19_3, L20_3)
      L18_3 = L4_1
      L18_3 = L18_3.yield
      L18_3()
      L19_3 = L14_3
      L18_3 = L14_3.waitForRender
      L18_3, L19_3 = L18_3(L19_3)
      L21_3 = L1_3
      L20_3 = L1_3.isCanceled
      L20_3 = L20_3(L21_3)
      if L20_3 then
        L20_3 = L19_1
        L21_3 = L20_3
        L20_3 = L20_3.warn
        L22_3 = "User cancelled operation during rendering photo "
        L23_3 = L13_3
        L22_3 = L22_3 .. L23_3
        L20_3(L21_3, L22_3)
        L20_3 = false
        L3_2 = L20_3
        break
      end
      if not L18_3 then
        L20_3 = L19_1
        L21_3 = L20_3
        L20_3 = L20_3.error
        L22_3 = string
        L22_3 = L22_3.format
        L23_3 = "Failed to render photo %d (%s): %s"
        L24_3 = L13_3
        L25_3 = L16_3
        L26_3 = L19_3 or L26_3
        if not L19_3 then
          L26_3 = "Unknown"
        end
        L22_3, L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3 = L22_3(L23_3, L24_3, L25_3, L26_3)
        L20_3(L21_3, L22_3, L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3)
        L20_3 = L35_1
        L21_3 = L35_1
        L21_3 = #L21_3
        L21_3 = L21_3 + 1
        L22_3 = string
        L22_3 = L22_3.format
        L23_3 = [[


Failed to render photo %d (%s). Error: %s
This photo will be skipped.]]
        L24_3 = L13_3
        L25_3 = L16_3
        L26_3 = L19_3 or L26_3
        if not L19_3 then
          L26_3 = "Unknown"
        end
        L22_3 = L22_3(L23_3, L24_3, L25_3, L26_3)
        L20_3[L21_3] = L22_3
        L20_3 = _G
        L20_3 = L20_3.activePhotosMeta
        L20_3 = L20_3[L13_3]
        if L20_3 then
          L20_3 = _G
          L20_3 = L20_3.activePhotosMeta
          L20_3 = L20_3[L13_3]
          L20_3.analysisStatus = "failed_render"
        end
        L20_3 = table
        L20_3 = L20_3.insert
        L21_3 = L5_2
        L22_3 = L13_3
        L20_3(L21_3, L22_3)
        L20_3 = false
        L3_2 = L20_3
        L20_3 = L5_3 - 1
        L7_3 = L7_3 + L20_3
        L21_3 = L8_3
        L22_3 = L7_3
        L23_3 = L6_3
        L21_3(L22_3, L23_3)
        L21_3 = L4_1
        L21_3 = L21_3.yield
        L21_3()
        L21_3 = L19_1
        L22_3 = L21_3
        L21_3 = L21_3.warn
        L23_3 = string
        L23_3 = L23_3.format
        L24_3 = "Skipped remaining %d steps for failed render photo %d"
        L25_3 = L20_3
        L26_3 = L13_3
        L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3 = L23_3(L24_3, L25_3, L26_3)
        L21_3(L22_3, L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3)
      else
        L7_3 = L7_3 + 1
        L20_3 = L8_3
        L21_3 = L7_3
        L22_3 = L6_3
        L20_3(L21_3, L22_3)
        L20_3 = L4_1
        L20_3 = L20_3.yield
        L20_3()
        L20_3 = L14_3.destinationPath
        L21_3 = L40_1
        L22_3 = L40_1
        L22_3 = #L22_3
        L22_3 = L22_3 + 1
        L21_3[L22_3] = L20_3
        L21_3 = L42_1
        if L21_3 then
          L21_3 = L54_1
          L22_3 = L20_3
          L23_3 = L17_3
          L24_3 = A1_2
          L25_3 = L13_3
          L26_3 = L4_3
          L27_3 = L1_3
          L28_3 = L7_3
          L29_3 = L6_3
          L30_3 = L8_3
          L21_3, L22_3 = L21_3(L22_3, L23_3, L24_3, L25_3, L26_3, L27_3, L28_3, L29_3, L30_3)
          L7_3 = L7_3 + L21_3
          L24_3 = L1_3
          L23_3 = L1_3.isCanceled
          L23_3 = L23_3(L24_3)
          if L23_3 then
            L23_3 = L19_1
            L24_3 = L23_3
            L23_3 = L23_3.warn
            L25_3 = "User cancelled operation during immediate analysis of photo "
            L26_3 = L13_3
            L25_3 = L25_3 .. L26_3
            L23_3(L24_3, L25_3)
            L23_3 = false
            L3_2 = L23_3
            break
          end
          if L22_3 == "failed" then
            L23_3 = L19_1
            L24_3 = L23_3
            L23_3 = L23_3.warn
            L25_3 = "Immediate analysis failed critically for photo "
            L26_3 = L13_3
            L23_3(L24_3, L25_3, L26_3)
            L23_3 = table
            L23_3 = L23_3.insert
            L24_3 = L5_2
            L25_3 = L13_3
            L23_3(L24_3, L25_3)
            L23_3 = false
            L3_2 = L23_3
          elseif L22_3 == "partial" then
            L23_3 = L19_1
            L24_3 = L23_3
            L23_3 = L23_3.warn
            L25_3 = "Immediate analysis partially succeeded for photo "
            L26_3 = L13_3
            L23_3(L24_3, L25_3, L26_3)
          end
        else
          L21_3 = table
          L21_3 = L21_3.insert
          L22_3 = L4_2
          L23_3 = {}
          L23_3.path = L20_3
          L23_3.id = L17_3
          L23_3.index = L13_3
          L21_3(L22_3, L23_3)
        end
      end
    end
    L10_3 = L42_1
    if not L10_3 then
      L10_3 = L3_2
      if L10_3 then
        L11_3 = L1_3
        L10_3 = L1_3.isCanceled
        L10_3 = L10_3(L11_3)
        if not L10_3 then
          L10_3 = ipairs
          L11_3 = L4_2
          L10_3, L11_3, L12_3 = L10_3(L11_3)
          for L13_3, L14_3 in L10_3, L11_3, L12_3 do
            L16_3 = L1_3
            L15_3 = L1_3.isCanceled
            L15_3 = L15_3(L16_3)
            if L15_3 then
              L15_3 = L19_1
              L16_3 = L15_3
              L15_3 = L15_3.warn
              L17_3 = "User cancelled operation during deferred analysis loop start."
              L15_3(L16_3, L17_3)
              L15_3 = false
              L3_2 = L15_3
              break
            end
            L15_3 = L14_3.index
            L16_3 = L54_1
            L17_3 = L14_3.path
            L18_3 = L14_3.id
            L19_3 = A1_2
            L20_3 = L15_3
            L21_3 = L4_3
            L22_3 = L1_3
            L23_3 = L7_3
            L24_3 = L6_3
            L25_3 = L8_3
            L16_3, L17_3 = L16_3(L17_3, L18_3, L19_3, L20_3, L21_3, L22_3, L23_3, L24_3, L25_3)
            L7_3 = L7_3 + L16_3
            L19_3 = L1_3
            L18_3 = L1_3.isCanceled
            L18_3 = L18_3(L19_3)
            if L18_3 then
              L18_3 = L19_1
              L19_3 = L18_3
              L18_3 = L18_3.warn
              L20_3 = "User cancelled operation during deferred analysis of photo "
              L21_3 = L15_3
              L20_3 = L20_3 .. L21_3
              L18_3(L19_3, L20_3)
              L18_3 = false
              L3_2 = L18_3
              break
            end
            if L17_3 == "failed" then
              L18_3 = L19_1
              L19_3 = L18_3
              L18_3 = L18_3.warn
              L20_3 = "Deferred analysis failed critically for photo "
              L21_3 = L15_3
              L18_3(L19_3, L20_3, L21_3)
              L18_3 = false
              L19_3 = ipairs
              L20_3 = L5_2
              L19_3, L20_3, L21_3 = L19_3(L20_3)
              for L22_3, L23_3 in L19_3, L20_3, L21_3 do
                if L23_3 == L15_3 then
                  L18_3 = true
                  break
                end
              end
              if not L18_3 then
                L19_3 = table
                L19_3 = L19_3.insert
                L20_3 = L5_2
                L21_3 = L15_3
                L19_3(L20_3, L21_3)
              end
              L19_3 = false
              L3_2 = L19_3
            elseif L17_3 == "partial" then
              L18_3 = L19_1
              L19_3 = L18_3
              L18_3 = L18_3.warn
              L20_3 = "Deferred analysis partially succeeded for photo "
              L21_3 = L15_3
              L18_3(L19_3, L20_3, L21_3)
            end
            L18_3 = L4_1
            L18_3 = L18_3.sleep
            L19_3 = 0.01
            L18_3(L19_3)
          end
        end
      end
    end
    L11_3 = L1_3
    L10_3 = L1_3.isCanceled
    L10_3 = L10_3(L11_3)
    if not L10_3 then
      L10_3 = L3_2
      if L10_3 then
        if L6_3 > L7_3 then
          L10_3 = L19_1
          L11_3 = L10_3
          L10_3 = L10_3.warn
          L12_3 = "Forcing progress to 100%."
          L10_3(L11_3, L12_3)
          L7_3 = L6_3
        end
        L10_3 = L8_3
        L11_3 = L7_3
        L12_3 = L6_3
        L10_3(L11_3, L12_3)
      else
        L10_3 = L19_1
        L11_3 = L10_3
        L10_3 = L10_3.warn
        L12_3 = "Final progress not forced to 100% due to errors or cancellation."
        L10_3(L11_3, L12_3)
      end
      L10_3 = L4_1
      L10_3 = L10_3.yield
      L10_3()
      L10_3 = A1_2
      L10_3 = L10_3.convertAsRoll
      if L10_3 == true and 1 < L4_3 then
        L10_3 = L3_2
        if L10_3 then
          L10_3 = {}
          L11_3 = {}
          L12_3 = 1
          L13_3 = L4_3
          L14_3 = 1
          for L15_3 = L12_3, L13_3, L14_3 do
            L16_3 = false
            L17_3 = ipairs
            L18_3 = L5_2
            L17_3, L18_3, L19_3 = L17_3(L18_3)
            for L20_3, L21_3 in L17_3, L18_3, L19_3 do
              if L15_3 == L21_3 then
                L16_3 = true
                break
              end
            end
            if not L16_3 then
              L17_3 = _G
              L17_3 = L17_3.activePhotosMeta
              L17_3 = L17_3[L15_3]
              if L17_3 then
                L17_3 = _G
                L17_3 = L17_3.activePhotosMeta
                L17_3 = L17_3[L15_3]
                L17_3 = L17_3.analysisStatus
                if L17_3 ~= "success" then
                  L17_3 = _G
                  L17_3 = L17_3.activePhotosMeta
                  L17_3 = L17_3[L15_3]
                  L17_3 = L17_3.analysisStatus
                end
                if L17_3 == "partial" then
                  L17_3 = A0_2
                  L17_3 = L17_3[L15_3]
                  if L17_3 then
                    L18_3 = table
                    L18_3 = L18_3.insert
                    L19_3 = L10_3
                    L20_3 = L17_3
                    L18_3(L19_3, L20_3)
                    L18_3 = table
                    L18_3 = L18_3.insert
                    L19_3 = L11_3
                    L20_3 = L15_3
                    L18_3(L19_3, L20_3)
                  end
                end
            end
            elseif not L16_3 then
              L17_3 = L19_1
              L18_3 = L17_3
              L17_3 = L17_3.warn
              L19_3 = "Photo "
              L20_3 = L15_3
              L21_3 = " did not fail critically but lacks success/partial metadata for roll inclusion."
              L19_3 = L19_3 .. L20_3 .. L21_3
              L17_3(L18_3, L19_3)
            end
          end
          L12_3 = #L10_3
          if 0 < L12_3 then
            L12_3 = "Roll ("
            L13_3 = #L10_3
            L14_3 = " frames)"
            L12_3 = L12_3 .. L13_3 .. L14_3
            L13_3 = A0_2
            L14_3 = L11_3[1]
            L13_3 = L13_3[L14_3]
            if L13_3 then
              L15_3 = L13_3
              L14_3 = L13_3.getRawMetadata
              L16_3 = "path"
              L14_3 = L14_3(L15_3, L16_3)
              if L14_3 then
                L15_3 = L6_1
                L15_3 = L15_3.parent
                L16_3 = L14_3
                L15_3 = L15_3(L16_3)
                if L15_3 then
                  L16_3 = L6_1
                  L16_3 = L16_3.leafName
                  L17_3 = L15_3
                  L16_3 = L16_3(L17_3)
                  if L16_3 and L16_3 ~= "" then
                    L17_3 = L16_3
                    L18_3 = " ("
                    L19_3 = #L10_3
                    L20_3 = " frames)"
                    L12_3 = L17_3 .. L18_3 .. L19_3 .. L20_3
                  else
                    L17_3 = L19_1
                    L18_3 = L17_3
                    L17_3 = L17_3.warn
                    L19_3 = "Could not determine folder name for roll naming."
                    L17_3(L18_3, L19_3)
                  end
                else
                  L16_3 = L19_1
                  L17_3 = L16_3
                  L16_3 = L16_3.warn
                  L18_3 = "Could not get parent path for roll naming."
                  L16_3(L17_3, L18_3)
                end
              else
                L15_3 = L19_1
                L16_3 = L15_3
                L15_3 = L15_3.warn
                L17_3 = "Could not get path metadata for roll naming."
                L15_3(L16_3, L17_3)
              end
            end
            L14_3 = "User"
            L15_3 = true
            L16_3 = true
            L17_3 = L18_1
            if L17_3 then
              L17_3 = L18_1
              L17_3 = L17_3.createRoll
              if L17_3 then
                L17_3 = L18_1
                L17_3 = L17_3.createRoll
                L18_3 = L10_3
                L19_3 = L12_3
                L20_3 = L14_3
                L21_3 = L15_3
                L22_3 = L16_3
                L17_3 = L17_3(L18_3, L19_3, L20_3, L21_3, L22_3)
                if L17_3 then
                  L18_3 = ipairs
                  L19_3 = L11_3
                  L18_3, L19_3, L20_3 = L18_3(L19_3)
                  for L21_3, L22_3 in L18_3, L19_3, L20_3 do
                    L23_3 = _G
                    L23_3 = L23_3.activePhotosMeta
                    L23_3 = L23_3[L22_3]
                    if L23_3 then
                      L23_3 = _G
                      L23_3 = L23_3.activePhotosMeta
                      L23_3 = L23_3[L22_3]
                      L23_3 = L23_3.settings
                      if L23_3 then
                        L23_3 = _G
                        L23_3 = L23_3.activePhotosMeta
                        L23_3 = L23_3[L22_3]
                        L23_3 = L23_3.settings
                        L23_3.rollAnalysis = L17_3
                      end
                    end
                  end
                else
                  L18_3 = L19_1
                  L19_3 = L18_3
                  L18_3 = L18_3.error
                  L20_3 = "NLPRolls.createRoll failed."
                  L18_3(L19_3, L20_3)
                  L18_3 = L35_1
                  L19_3 = L35_1
                  L19_3 = #L19_3
                  L19_3 = L19_3 + 1
                  L18_3[L19_3] = [[


Roll Analysis failed to create.]]
                end
            end
            else
              L17_3 = L19_1
              L18_3 = L17_3
              L17_3 = L17_3.error
              L19_3 = "NLPRolls.createRoll missing."
              L17_3(L18_3, L19_3)
              L17_3 = L35_1
              L18_3 = L35_1
              L18_3 = #L18_3
              L18_3 = L18_3 + 1
              L17_3[L18_3] = [[


Internal Error: Roll Analysis function missing.]]
            end
          else
            L12_3 = L19_1
            L13_3 = L12_3
            L12_3 = L12_3.warn
            L14_3 = "Roll Analysis skipped: No photos succeeded sufficiently."
            L12_3(L13_3, L14_3)
            L12_3 = L5_2
            L12_3 = #L12_3
            if L4_3 > L12_3 then
              L12_3 = L35_1
              L13_3 = L35_1
              L13_3 = #L13_3
              L13_3 = L13_3 + 1
              L12_3[L13_3] = [[


Roll Analysis skipped: No photos were successfully analyzed.]]
            end
          end
      end
      else
        L10_3 = A1_2
        L10_3 = L10_3.convertAsRoll
        if L10_3 == true and L4_3 <= 1 then
          L10_3 = L19_1
          L11_3 = L10_3
          L10_3 = L10_3.info
          L12_3 = "Roll Analysis skipped: Fewer than 2 photos."
          L10_3(L11_3, L12_3)
        else
          L10_3 = A1_2
          L10_3 = L10_3.convertAsRoll
          if L10_3 == true then
            L10_3 = L3_2
            if not L10_3 then
              L10_3 = L19_1
              L11_3 = L10_3
              L10_3 = L10_3.warn
              L12_3 = "Roll Analysis skipped due to errors/cancel."
              L10_3(L11_3, L12_3)
            end
          end
        end
      end
      L10_3 = L4_1
      L10_3 = L10_3.yield
      L10_3()
      L10_3 = L3_2
      if L10_3 then
        L10_3 = L36_1
        if L10_3 then
          L10_3 = L36_1
          L11_3 = {}
          L12_3 = A1_2
          L12_3 = L12_3.source
          L11_3.source = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.filmBorder
          L11_3.filmBorder = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.saturation
          L11_3.saturation = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.colorModel
          L11_3.colorModel = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.previewBuffer
          L11_3.previewBuffer = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.convertAsRoll
          L11_3.convertAsRoll = L12_3
          L12_3 = A1_2
          L12_3 = L12_3.imageType
          L11_3.imageType = L12_3
          L10_3.conversionSettings = L11_3
          L10_3 = pcall
          
          function L11_3()
            local L0_4, L1_4
            L0_4 = L36_1
            L1_4 = L36_1
            L1_4 = L1_4.conversionSettings
            L0_4.conversionSettings = L1_4
          end
          
          L10_3, L11_3 = L10_3(L11_3)
          if not L10_3 then
            L12_3 = L19_1
            L13_3 = L12_3
            L12_3 = L12_3.error
            L14_3 = "Failed to save preferences: "
            L15_3 = tostring
            L16_3 = L11_3
            L15_3 = L15_3(L16_3)
            L14_3 = L14_3 .. L15_3
            L12_3(L13_3, L14_3)
          end
        else
          L10_3 = L19_1
          L11_3 = L10_3
          L10_3 = L10_3.error
          L12_3 = "Cannot save preferences: _G.prefs not available."
          L10_3(L11_3, L12_3)
        end
      else
        L10_3 = L19_1
        L11_3 = L10_3
        L10_3 = L10_3.info
        L12_3 = "Skipping preference save due to errors/cancel."
        L10_3(L11_3, L12_3)
      end
    else
      L10_3 = L19_1
      L11_3 = L10_3
      L10_3 = L10_3.warn
      L12_3 = "Finalization skipped due to cancellation."
      L10_3(L11_3, L12_3)
      L10_3 = false
      L3_2 = L10_3
    end
    L11_3 = L1_3
    L10_3 = L1_3.done
    L10_3(L11_3)
  end
  
  L6_2(L7_2, L8_2)
  L6_2 = L35_1
  L6_2 = #L6_2
  if 0 < L6_2 then
    L6_2 = L37_1
    if L6_2 == true then
      L6_2 = L19_1
      L7_2 = L6_2
      L6_2 = L6_2.warn
      L8_2 = "Errors occurred during processing. Displaying error dialog."
      L6_2(L7_2, L8_2)
      L6_2 = L55_1
      if L6_2 then
        L6_2 = L55_1
        L6_2()
        L6_2 = {}
        L35_1 = L6_2
      else
        L6_2 = L2_1
        L6_2 = L6_2.showError
        L7_2 = "Processing completed with errors. Check log."
        L6_2(L7_2)
      end
      L6_2 = NLPDialogs
      if L6_2 then
        L6_2 = NLPDialogs
        L6_2 = L6_2.mainDialog
        if L6_2 then
          L6_2 = NLPDialogs
          L6_2 = L6_2.mainDialog
          L6_2()
      end
      else
        L6_2 = L19_1
        L7_2 = L6_2
        L6_2 = L6_2.error
        L8_2 = "NLPDialogs.mainDialog missing!"
        L6_2(L7_2, L8_2)
      end
  end
  else
    L6_2 = NLPDialogs
    if L6_2 then
      L6_2 = NLPDialogs
      L6_2 = L6_2.mainDialog
      if L6_2 then
        L6_2 = NLPDialogs
        L6_2 = L6_2.mainDialog
        L6_2()
    end
    else
      L6_2 = L19_1
      L7_2 = L6_2
      L6_2 = L6_2.error
      L8_2 = "NLPDialogs.mainDialog missing!"
      L6_2(L7_2, L8_2)
    end
  end
  L6_2 = _G
  L6_2.testScenario = nil
  L6_2 = _G
  L6_2.testFailPhotoIndex = nil
end

L57_1.processPhotoV3 = L58_1
L57_1 = NLPImageProcessor
return L57_1
