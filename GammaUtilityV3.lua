local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrApplication"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrApplicationView"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrBinding"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrFunctionContext"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrDialogs"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrView"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrTasks"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrLogger"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrPathUtils"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrFileUtils"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrProgressScope"
L11_1 = L11_1(L12_1)
L12_1 = import
L13_1 = "LrDevelopController"
L12_1 = L12_1(L13_1)
L13_1 = import
L14_1 = "LrColor"
L13_1 = L13_1(L14_1)
L14_1 = import
L15_1 = "LrExportSession"
L14_1 = L14_1(L15_1)
L15_1 = import
L16_1 = "LrProgressScope"
L15_1 = L15_1(L16_1)
L16_1 = require
L17_1 = "UtilV3"
L16_1 = L16_1(L17_1)
L17_1 = require
L18_1 = "ExifV3"
L17_1 = L17_1(L18_1)
L18_1 = import
L19_1 = "LrUUID"
L18_1 = L18_1(L19_1)
L19_1 = L9_1.getStandardFilePath
L20_1 = "temp"
L19_1 = L19_1(L20_1)
L20_1 = L8_1
L21_1 = "Negative"
L20_1 = L20_1(L21_1)
L21_1 = _G
L21_1 = L21_1.dev
if L21_1 == true then
  L22_1 = L20_1
  L21_1 = L20_1.enable
  L23_1 = "logfile"
  L21_1(L22_1, L23_1)
else
  L22_1 = L20_1
  L21_1 = L20_1.disable
  L21_1(L22_1)
end
L21_1 = "\""
L22_1 = "balance-channels"
L23_1 = {}
L24_1 = {}
L25_1 = "auto"
L26_1 = "auto"
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.auto_gamma = L24_1
L24_1 = {}
L25_1 = 1
L26_1 = 2.2
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.linear = L24_1
L24_1 = {}
L25_1 = 1.8
L26_1 = 2.2
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.flextight = L24_1
L24_1 = {}
L25_1 = 1.8
L26_1 = 2.2
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.epsonScan = L24_1
L24_1 = {}
L25_1 = 1
L26_1 = 2.2
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.pakonPPRC = L24_1
L24_1 = {}
L25_1 = 1
L26_1 = 2.2
L24_1[1] = L25_1
L24_1[2] = L26_1
L23_1.custom = L24_1
L24_1 = L9_1.child
L25_1 = _PLUGIN
L25_1 = L25_1.path
L26_1 = "mac/magick"
L24_1 = L24_1(L25_1, L26_1)
L25_1 = WIN_ENV
if L25_1 == true then
  L25_1 = L9_1.child
  L26_1 = _PLUGIN
  L26_1 = L26_1.path
  L27_1 = "win\\convert.exe"
  L25_1 = L25_1(L26_1, L27_1)
  L24_1 = L25_1
end

function L25_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = WIN_ENV
  if L1_2 then
    L1_2 = "\""
    L2_2 = A0_2
    L3_2 = "\""
    L1_2 = L1_2 .. L2_2 .. L3_2
    return L1_2
  else
    return A0_2
  end
end

function L26_1(A0_2)
  local L1_2
  L1_2 = MAC_ENV
  if L1_2 then
    return A0_2
  end
end

function L27_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = "\""
  L2_2 = A0_2
  L3_2 = "\""
  L1_2 = L1_2 .. L2_2 .. L3_2
  return L1_2
end

function L28_1(A0_2)
  local L1_2, L2_2
  if 0 <= A0_2 then
    L1_2 = math
    L1_2 = L1_2.floor
    L2_2 = A0_2 + 0.5
    return L1_2(L2_2)
  else
    L1_2 = math
    L1_2 = L1_2.ceil
    L2_2 = A0_2 - 0.5
    return L1_2(L2_2)
  end
end

round = L28_1
L28_1 = L1_1.activeCatalog
L28_1 = L28_1()
L29_1 = {}
L29_1 = L28_1.targetPhotos
L30_1 = 1
L31_1 = #L29_1
L32_1 = 1
for L33_1 = L30_1, L31_1, L32_1 do
  L34_1 = L29_1[L33_1]
  L35_1 = L34_1
  L34_1 = L34_1.saveMetadata
  L34_1(L35_1)
end
L30_1 = L7_1.startAsyncTask

function L31_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2
  L0_2 = 1
  L1_2 = L29_1
  L1_2 = #L1_2
  L2_2 = 1
  for L3_2 = L0_2, L1_2, L2_2 do
    L4_2 = L29_1
    L4_2 = L4_2[L3_2]
    L5_2 = L4_2
    L4_2 = L4_2.getDevelopSettings
    L4_2 = L4_2(L5_2)
  end
end

L30_1(L31_1)

function L30_1(A0_2, A1_2)
  local L2_2, L3_2
  if A1_2 == "AB" then
    return A0_2
  elseif A1_2 == "BC" then
    L2_2 = {}
    L3_2 = A0_2.bottom
    L2_2.right = L3_2
    L3_2 = A0_2.left
    L3_2 = 1 - L3_2
    L2_2.bottom = L3_2
    L3_2 = A0_2.top
    L2_2.left = L3_2
    L3_2 = A0_2.right
    L3_2 = 1 - L3_2
    L2_2.top = L3_2
    L3_2 = A0_2.angle
    L2_2.angle = L3_2
    return L2_2
  elseif A1_2 == "CD" then
    L2_2 = {}
    L3_2 = A0_2.top
    L3_2 = 1 - L3_2
    L2_2.bottom = L3_2
    L3_2 = A0_2.right
    L3_2 = 1 - L3_2
    L2_2.left = L3_2
    L3_2 = A0_2.bottom
    L3_2 = 1 - L3_2
    L2_2.top = L3_2
    L3_2 = A0_2.left
    L3_2 = 1 - L3_2
    L2_2.right = L3_2
    L3_2 = A0_2.angle
    L2_2.angle = L3_2
    return L2_2
  elseif A1_2 == "DA" then
    L2_2 = {}
    L3_2 = A0_2.bottom
    L3_2 = 1 - L3_2
    L2_2.left = L3_2
    L3_2 = A0_2.left
    L2_2.top = L3_2
    L3_2 = A0_2.top
    L3_2 = 1 - L3_2
    L2_2.right = L3_2
    L3_2 = A0_2.right
    L2_2.bottom = L3_2
    L3_2 = A0_2.angle
    L2_2.angle = L3_2
    return L2_2
  end
end

rotateCropForOrientation = L30_1

function L30_1(A0_2)
  local L1_2, L2_2
  L1_2 = L23_1
  L2_2 = A0_2.gammaPreset
  L1_2 = L1_2[L2_2]
  L1_2 = L1_2[1]
  A0_2.gammaInput = L1_2
  L1_2 = L23_1
  L2_2 = A0_2.gammaPreset
  L1_2 = L1_2[L2_2]
  L1_2 = L1_2[2]
  A0_2.gammaOutput = L1_2
end

function L31_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = A0_2.gammaInput
  L2_2 = A0_2.gammaOutput
  L3_2 = 1 / L1_2
  L4_2 = L4_1
  L4_2 = L4_2.postAsyncTaskWithContext
  L5_2 = "processImagesWithSettings"
  
  function L6_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
    L1_3 = L29_1
    L1_3 = #L1_3
    if 1 < L1_3 then
      L1_3 = "Preparing "
      L2_3 = L29_1
      L2_3 = #L2_3
      L3_3 = " Negatives..."
      L1_3 = L1_3 .. L2_3 .. L3_3
      if L1_3 then
        goto lbl_13
      end
    end
    L1_3 = "Preparing Negative..."
    ::lbl_13::
    L2_3 = L15_1
    L3_3 = {}
    L3_3.title = L1_3
    L3_3.functionContext = A0_3
    L2_3 = L2_3(L3_3)
    L3_3 = 0
    L4_3 = 1
    L5_3 = L29_1
    L5_3 = #L5_3
    L6_3 = 1
    for L7_3 = L4_3, L5_3, L6_3 do
      L8_3 = L4_1
      L8_3 = L8_3.callWithContext
      L9_3 = "processImage"
      
      function L10_3(A0_4)
        local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4, L15_4, L16_4, L17_4, L18_4, L19_4, L20_4, L21_4, L22_4, L23_4, L24_4, L25_4, L26_4, L27_4, L28_4, L29_4, L30_4, L31_4, L32_4, L33_4, L34_4, L35_4, L36_4, L37_4, L38_4, L39_4, L40_4, L41_4, L42_4, L43_4, L44_4, L45_4, L46_4, L47_4, L48_4, L49_4, L50_4, L51_4, L52_4
        L1_4 = L18_1
        L1_4 = L1_4.generateUUID
        L1_4 = L1_4()
        L2_4 = L9_1
        L2_4 = L2_4.child
        L3_4 = L19_1
        L4_4 = L1_4
        L5_4 = ".txt"
        L4_4 = L4_4 .. L5_4
        L2_4 = L2_4(L3_4, L4_4)
        L4_4 = A0_4
        L3_4 = A0_4.addCleanupHandler
        
        function L5_4()
          local L0_5, L1_5
          L0_5 = L10_1
          L0_5 = L0_5.exists
          L1_5 = L2_4
          L0_5 = L0_5(L1_5)
          if L0_5 then
            L0_5 = L10_1
            L0_5 = L0_5.delete
            L1_5 = L2_4
            L0_5(L1_5)
          end
        end
        
        L3_4(L4_4, L5_4)
        L3_4 = L7_3
        L4_4 = L29_1
        L4_4 = #L4_4
        L3_4 = L3_4 / L4_4
        L4_4 = L15_1
        L5_4 = {}
        L6_4 = L2_3
        L5_4.parent = L6_4
        L5_4.parentEndRange = L3_4
        L6_4 = "Processing Image "
        L7_4 = L7_3
        L6_4 = L6_4 .. L7_4
        L5_4.caption = L6_4
        L5_4.functionContext = A0_4
        L4_4 = L4_4(L5_4)
        L4_4.setPortionComplete = 0.33
        L5_4 = L29_1
        L6_4 = L7_3
        L5_4 = L5_4[L6_4]
        L7_4 = L5_4
        L6_4 = L5_4.getRawMetadata
        L8_4 = "path"
        L6_4 = L6_4(L7_4, L8_4)
        L7_4 = L9_1
        L7_4 = L7_4.parent
        L8_4 = L6_4
        L7_4 = L7_4(L8_4)
        L8_4 = L9_1
        L8_4 = L8_4.leafName
        L9_4 = L6_4
        L8_4 = L8_4(L9_4)
        L10_4 = L5_4
        L9_4 = L5_4.getDevelopSettings
        L9_4 = L9_4(L10_4)
        L10_4 = L9_4.orientation
        L11_4 = L9_4.CropAngle
        L11_4 = L11_4 * -1
        L12_4 = L9_4.CropBottom
        L13_4 = L9_4.CropLeft
        L14_4 = L9_4.CropRight
        L15_4 = L9_4.CropTop
        L17_4 = L5_4
        L16_4 = L5_4.getRawMetadata
        L18_4 = "dimensions"
        L16_4 = L16_4(L17_4, L18_4)
        L18_4 = L5_4
        L17_4 = L5_4.getRawMetadata
        L19_4 = "isCropped"
        L17_4 = L17_4(L18_4, L19_4)
        L18_4 = L16_4.height
        L19_4 = L16_4.width
        L20_4 = ""
        L21_4 = ""
        L22_4 = ""
        if L17_4 == true then
          L23_4 = A0_2
          L23_4 = L23_4.isCropChecked
          if L23_4 == true then
            L23_4 = round
            L24_4 = L13_4 * L19_4
            L23_4 = L23_4(L24_4)
            L24_4 = round
            L25_4 = L15_4 * L18_4
            L24_4 = L24_4(L25_4)
            L25_4 = round
            L26_4 = L14_4 - L13_4
            L26_4 = L26_4 * L19_4
            L25_4 = L25_4(L26_4)
            L26_4 = round
            L27_4 = L12_4 - L15_4
            L27_4 = L27_4 * L18_4
            L26_4 = L26_4(L27_4)
            L27_4 = " -crop "
            L28_4 = L25_4
            L29_4 = "x"
            L30_4 = L26_4
            L31_4 = "+"
            L32_4 = L23_4
            L33_4 = "+"
            L34_4 = L24_4
            L35_4 = " +repage "
            L22_4 = L27_4 .. L28_4 .. L29_4 .. L30_4 .. L31_4 .. L32_4 .. L33_4 .. L34_4 .. L35_4
            L27_4 = L21_4
            L28_4 = L22_4
            L20_4 = L27_4 .. L28_4
          end
        end
        L23_4 = ""
        L24_4 = ""
        L25_4 = ""
        L26_4 = ""
        L27_4 = ""
        L28_4 = ""
        L29_4 = ""
        L30_4 = A0_2
        L30_4 = L30_4.gammaPreset
        if L30_4 == "auto_gamma" then
          L26_4 = " +channel -auto-gamma "
          L23_4 = "-auto-gamma"
        else
          L30_4 = " -gamma "
          L31_4 = L3_2
          L32_4 = "  "
          L26_4 = L30_4 .. L31_4 .. L32_4
          L30_4 = "-gamma-"
          L31_4 = L1_2
          L23_4 = L30_4 .. L31_4
        end
        L30_4 = nil
        L31_4 = A0_2
        L31_4 = L31_4.saveOption
        if L31_4 == "stack" then
          L30_4 = L7_4
        else
          L31_4 = A0_2
          L31_4 = L31_4.saveOption
          if L31_4 == "subfolder" then
            L31_4 = L9_1
            L31_4 = L31_4.child
            L32_4 = L7_4
            L33_4 = "gamma"
            L31_4 = L31_4(L32_4, L33_4)
            L30_4 = L31_4
            L31_4 = L10_1
            L31_4 = L31_4.exists
            L32_4 = L30_4
            L31_4 = L31_4(L32_4)
            if L31_4 ~= true then
              L31_4 = L10_1
              L31_4 = L31_4.createDirectory
              L32_4 = L30_4
              L31_4(L32_4)
            end
          end
        end
        L31_4 = L9_1
        L31_4 = L31_4.removeExtension
        L32_4 = L8_4
        L31_4 = L31_4(L32_4)
        L32_4 = L23_4
        L33_4 = "."
        L34_4 = L9_1
        L34_4 = L34_4.extension
        L35_4 = L8_4
        L34_4 = L34_4(L35_4)
        L24_4 = L31_4 .. L32_4 .. L33_4 .. L34_4
        L31_4 = L10_1
        L31_4 = L31_4.chooseUniqueFileName
        L32_4 = L9_1
        L32_4 = L32_4.child
        L33_4 = L30_4
        L34_4 = L24_4
        L32_4, L33_4, L34_4, L35_4, L36_4, L37_4, L38_4, L39_4, L40_4, L41_4, L42_4, L43_4, L44_4, L45_4, L46_4, L47_4, L48_4, L49_4, L50_4, L51_4, L52_4 = L32_4(L33_4, L34_4)
        L31_4 = L31_4(L32_4, L33_4, L34_4, L35_4, L36_4, L37_4, L38_4, L39_4, L40_4, L41_4, L42_4, L43_4, L44_4, L45_4, L46_4, L47_4, L48_4, L49_4, L50_4, L51_4, L52_4)
        L25_4 = L31_4
        L31_4 = L27_1
        L32_4 = L24_1
        L31_4 = L31_4(L32_4)
        L32_4 = " -quiet "
        L33_4 = L21_1
        L34_4 = L6_4
        L35_4 = "[0]"
        L36_4 = L21_1
        L37_4 = " -flatten -alpha off -gamma "
        L38_4 = L3_2
        L39_4 = " -separate -format \"%[max]\\n\" info: >> "
        L40_4 = L21_1
        L41_4 = L2_4
        L42_4 = L21_1
        L31_4 = L31_4 .. L32_4 .. L33_4 .. L34_4 .. L35_4 .. L36_4 .. L37_4 .. L38_4 .. L39_4 .. L40_4 .. L41_4 .. L42_4
        L32_4 = L16_1
        L32_4 = L32_4.safeExecute
        L33_4 = L31_4
        L34_4 = error_only
        L35_4 = " analyzing tiff color multipliers"
        L32_4(L33_4, L34_4, L35_4)
        L32_4 = io
        L32_4 = L32_4.open
        L33_4 = L2_4
        L34_4 = "r"
        L32_4 = L32_4(L33_4, L34_4)
        L34_4 = L32_4
        L33_4 = L32_4.read
        L35_4 = "*l"
        L33_4 = L33_4(L34_4, L35_4)
        L35_4 = L32_4
        L34_4 = L32_4.read
        L36_4 = "*l"
        L34_4 = L34_4(L35_4, L36_4)
        L36_4 = L32_4
        L35_4 = L32_4.read
        L37_4 = "*l"
        L35_4 = L35_4(L36_4, L37_4)
        L37_4 = L32_4
        L36_4 = L32_4.close
        L36_4(L37_4)
        L36_4 = L27_1
        L37_4 = L24_1
        L36_4 = L36_4(L37_4)
        L37_4 = " -quiet "
        L38_4 = L21_1
        L39_4 = L6_4
        L40_4 = "[0]"
        L41_4 = L21_1
        L42_4 = " -flatten -alpha off +channel -gamma "
        L43_4 = L3_2
        L44_4 = " -channel R -level 0,"
        L45_4 = L33_4
        L46_4 = " -channel G -level 0,"
        L47_4 = L34_4
        L48_4 = " -channel B -level 0,"
        L49_4 = L35_4
        L50_4 = " +channel -gamma 2.2 "
        L51_4 = L27_1
        L52_4 = L25_4
        L51_4 = L51_4(L52_4)
        L36_4 = L36_4 .. L37_4 .. L38_4 .. L39_4 .. L40_4 .. L41_4 .. L42_4 .. L43_4 .. L44_4 .. L45_4 .. L46_4 .. L47_4 .. L48_4 .. L49_4 .. L50_4 .. L51_4
        L37_4 = L16_1
        L37_4 = L37_4.safeExecute
        L38_4 = L36_4
        L39_4 = "error_only"
        L40_4 = "process a tiff using gamma and color multipliers."
        L37_4, L38_4 = L37_4(L38_4, L39_4, L40_4)
        L39_4 = MAC_ENV
        if L39_4 then
          L39_4 = " -m -overwrite_original -E "
          if L39_4 then
            goto lbl_238
          end
        end
        L39_4 = " -charset Latin -m -overwrite_original -E "
        ::lbl_238::
        L40_4 = " -tagsFromFile "
        L41_4 = L21_1
        L42_4 = L6_4
        L43_4 = L21_1
        L40_4 = L40_4 .. L41_4 .. L42_4 .. L43_4
        L41_4 = L17_1
        L41_4 = L41_4.execute
        L42_4 = L39_4
        L43_4 = L40_4
        L44_4 = {}
        L45_4 = L25_4
        L44_4[1] = L45_4
        L41_4(L42_4, L43_4, L44_4)
        L41_4 = L28_1
        L42_4 = L41_4
        L41_4 = L41_4.withWriteAccessDo
        L43_4 = "Add gamma corrected photo"
        
        function L44_4(A0_5)
          local L1_5, L2_5, L3_5, L4_5, L5_5
          L1_5 = L28_1
          L2_5 = L1_5
          L1_5 = L1_5.findPhotoByPath
          L3_5 = L25_4
          L1_5 = L1_5(L2_5, L3_5)
          L2_5 = A0_2
          L2_5 = L2_5.saveOption
          if L2_5 == "stack" then
            L2_5 = L28_1
            L3_5 = L2_5
            L2_5 = L2_5.addPhoto
            L4_5 = L25_4
            L5_5 = L5_4
            L2_5(L3_5, L4_5, L5_5)
          else
            L2_5 = A0_2
            L2_5 = L2_5.saveOption
            if L2_5 == "subfolder" then
              L2_5 = L28_1
              L3_5 = L2_5
              L2_5 = L2_5.addPhoto
              L4_5 = L25_4
              L2_5(L3_5, L4_5)
            end
          end
        end
        
        L45_4 = {}
        L45_4.timeout = 5
        L45_4.asynchronous = true
        L41_4(L42_4, L43_4, L44_4, L45_4)
      end
      
      L8_3(L9_3, L10_3)
    end
  end
  
  L4_2(L5_2, L6_2)
end

function L32_1()
  local L0_2, L1_2
  L0_2 = L7_1
  L0_2 = L0_2.startAsyncTask
  
  function L1_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3
    L0_3 = 0
    L1_3 = 1
    L2_3 = L29_1
    L2_3 = #L2_3
    L3_3 = 1
    for L4_3 = L1_3, L2_3, L3_3 do
      L5_3 = L29_1
      L5_3 = L5_3[L4_3]
      L7_3 = L5_3
      L6_3 = L5_3.getRawMetadata
      L8_3 = "fileFormat"
      L6_3 = L6_3(L7_3, L8_3)
      L8_3 = L5_3
      L7_3 = L5_3.getRawMetadata
      L9_3 = "fileSize"
      L7_3 = L7_3(L8_3, L9_3)
      L0_3 = L0_3 + L7_3
      if L6_3 ~= "TIFF" and L6_3 ~= "JPG" then
        L8_3 = "This utility only works with TIF files."
        L9_3 = L5_1
        L9_3 = L9_3.message
        L10_3 = L8_3
        L11_3 = nil
        L12_3 = "critical"
        L9_3(L10_3, L11_3, L12_3)
        return
      end
    end
    L1_3 = round
    L2_3 = L0_3 * 1.0E-6
    L1_3 = L1_3(L2_3)
    L2_3 = L29_1
    L2_3 = #L2_3
    L3_3 = " photo(s), "
    L4_3 = L1_3
    L5_3 = " MBs"
    L2_3 = L2_3 .. L3_3 .. L4_3 .. L5_3
    L3_3 = L4_1
    L3_3 = L3_3.callWithContext
    L4_3 = "gammaDialog"
    
    function L5_3(A0_4)
      local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4, L15_4, L16_4, L17_4, L18_4, L19_4, L20_4, L21_4
      L1_4 = L6_1
      L1_4 = L1_4.osFactory
      L1_4 = L1_4()
      L2_4 = L3_1
      L2_4 = L2_4.makePropertyTable
      L3_4 = A0_4
      L2_4 = L2_4(L3_4)
      L3_4 = L3_1
      L3_4 = L3_4.makePropertyTable
      L4_4 = A0_4
      L3_4 = L3_4(L4_4)
      L4_4 = {}
      L5_4 = "gammaPreset"
      L6_4 = "gammaInput"
      L7_4 = "gammaOutput"
      L8_4 = "isColorBalanced"
      L9_4 = "isNormalized"
      L10_4 = "saveOption"
      L11_4 = "borderBuffer"
      L4_4[1] = L5_4
      L4_4[2] = L6_4
      L4_4[3] = L7_4
      L4_4[4] = L8_4
      L4_4[5] = L9_4
      L4_4[6] = L10_4
      L4_4[7] = L11_4
      L2_4.gammaPreset = "linear"
      L2_4.gammaInput = 1
      L2_4.gammaOutput = 2.2
      L2_4.isColorBalanced = false
      L2_4.isNormalized = false
      L2_4.saveOption = "stack"
      L2_4.isCropChecked = false
      L2_4.borderBuffer = false
      L6_4 = L2_4
      L5_4 = L2_4.addObserver
      L7_4 = "gammaPreset"
      L8_4 = L30_1
      L5_4(L6_4, L7_4, L8_4)
      L3_4.customGamma = true
      L6_4 = L1_4
      L5_4 = L1_4.column
      L7_4 = {}
      L7_4.bind_to_object = L2_4
      L7_4.fill_horizontal = 1
      L7_4.spacing = 2
      L9_4 = L1_4
      L8_4 = L1_4.static_text
      L10_4 = {}
      L10_4.title = "Gamma Settings:"
      L8_4 = L8_4(L9_4, L10_4)
      L10_4 = L1_4
      L9_4 = L1_4.popup_menu
      L11_4 = {}
      L12_4 = L6_1
      L12_4 = L12_4.bind
      L13_4 = "gammaPreset"
      L12_4 = L12_4(L13_4)
      L11_4.value = L12_4
      L12_4 = {}
      L13_4 = {}
      L13_4.title = "Linear"
      L13_4.value = "linear"
      L14_4 = {}
      L14_4.title = "Flextight MAC"
      L14_4.value = "flextight"
      L15_4 = {}
      L15_4.title = "EpsonScan Mac"
      L15_4.value = "epsonScan"
      L16_4 = {}
      L16_4.title = "Pakon PPRC"
      L16_4.value = "pakonPPRC"
      L17_4 = {}
      L17_4.title = "--CUSTOM--"
      L17_4.value = "custom"
      L12_4[1] = L13_4
      L12_4[2] = L14_4
      L12_4[3] = L15_4
      L12_4[4] = L16_4
      L12_4[5] = L17_4
      L11_4.items = L12_4
      L12_4 = L6_1
      L12_4 = L12_4.share
      L13_4 = "popup_menu_conversion_width"
      L12_4 = L12_4(L13_4)
      L11_4.width = L12_4
      L9_4 = L9_4(L10_4, L11_4)
      L11_4 = L1_4
      L10_4 = L1_4.spacer
      L12_4 = {}
      L12_4.height = 10
      L10_4 = L10_4(L11_4, L12_4)
      L12_4 = L1_4
      L11_4 = L1_4.row
      L13_4 = {}
      L15_4 = L1_4
      L14_4 = L1_4.static_text
      L16_4 = {}
      L16_4.title = "Input Gamma: "
      L17_4 = L6_1
      L17_4 = L17_4.share
      L18_4 = "input_gamma_width"
      L17_4 = L17_4(L18_4)
      L16_4.width = L17_4
      L14_4 = L14_4(L15_4, L16_4)
      L16_4 = L1_4
      L15_4 = L1_4.edit_field
      L17_4 = {}
      L18_4 = L3_1
      L18_4 = L18_4.keyEquals
      L19_4 = "gammaPreset"
      L20_4 = "custom"
      L21_4 = L2_4
      L18_4 = L18_4(L19_4, L20_4, L21_4)
      L17_4.enabled = L18_4
      L17_4.immediate = true
      L18_4 = L6_1
      L18_4 = L18_4.bind
      L19_4 = "gammaInput"
      L18_4 = L18_4(L19_4)
      L17_4.value = L18_4
      L17_4.min = 0.1
      L17_4.max = 5
      L17_4.precision = 3
      L17_4.width_in_digits = 6
      L17_4.increment = 0.1
      L17_4.alignment = "right"
      L15_4, L16_4, L17_4, L18_4, L19_4, L20_4, L21_4 = L15_4(L16_4, L17_4)
      L13_4[1] = L14_4
      L13_4[2] = L15_4
      L13_4[3] = L16_4
      L13_4[4] = L17_4
      L13_4[5] = L18_4
      L13_4[6] = L19_4
      L13_4[7] = L20_4
      L13_4[8] = L21_4
      L11_4 = L11_4(L12_4, L13_4)
      L13_4 = L1_4
      L12_4 = L1_4.separator
      L14_4 = {}
      L14_4.fill_horizontal = 1
      L14_4.height = 10
      L12_4 = L12_4(L13_4, L14_4)
      L14_4 = L1_4
      L13_4 = L1_4.static_text
      L15_4 = {}
      L15_4.fill_horizontal = 1
      L15_4.alignment = "center"
      L15_4.size = "small"
      L16_4 = L2_3
      L15_4.title = L16_4
      L13_4 = L13_4(L14_4, L15_4)
      L15_4 = L1_4
      L14_4 = L1_4.popup_menu
      L16_4 = {}
      L17_4 = L6_1
      L17_4 = L17_4.bind
      L18_4 = "saveOption"
      L17_4 = L17_4(L18_4)
      L16_4.value = L17_4
      L17_4 = {}
      L18_4 = {}
      L18_4.title = "Stack with Original"
      L18_4.value = "stack"
      L19_4 = {}
      L19_4.title = "Save to Subfolder"
      L19_4.value = "subfolder"
      L17_4[1] = L18_4
      L17_4[2] = L19_4
      L16_4.items = L17_4
      L17_4 = L6_1
      L17_4 = L17_4.share
      L18_4 = "popup_menu_conversion_width"
      L17_4 = L17_4(L18_4)
      L16_4.width = L17_4
      L14_4, L15_4, L16_4, L17_4, L18_4, L19_4, L20_4, L21_4 = L14_4(L15_4, L16_4)
      L7_4[1] = L8_4
      L7_4[2] = L9_4
      L7_4[3] = L10_4
      L7_4[4] = L11_4
      L7_4[5] = L12_4
      L7_4[6] = L13_4
      L7_4[7] = L14_4
      L7_4[8] = L15_4
      L7_4[9] = L16_4
      L7_4[10] = L17_4
      L7_4[11] = L18_4
      L7_4[12] = L19_4
      L7_4[13] = L20_4
      L7_4[14] = L21_4
      L5_4 = L5_4(L6_4, L7_4)
      L6_4 = "TIF Gamma Utility"
      L7_4 = "Make"
      L8_4 = L5_1
      L8_4 = L8_4.presentModalDialog
      L9_4 = {}
      L9_4.title = L6_4
      L9_4.contents = L5_4
      L9_4.actionVerb = L7_4
      L9_4.save_frame = "GammaUtilityFramePosition"
      L8_4 = L8_4(L9_4)
      if L8_4 == "ok" then
        L9_4 = L31_1
        L10_4 = L2_4
        L9_4(L10_4)
      end
    end
    
    L3_3(L4_3, L5_3)
  end
  
  L0_2(L1_2)
end

L33_1 = L32_1
L33_1()
