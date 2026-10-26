local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1, L36_1
L0_1 = import
L1_1 = "LrLogger"
L0_1 = L0_1(L1_1)
L1_1 = require
L2_1 = "UtilV3"
L1_1 = L1_1(L2_1)
L2_1 = require
L3_1 = "inspect"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrUUID"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrDialogs"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrView"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrBinding"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrFunctionContext"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrApplication"
L8_1 = L8_1(L9_1)
L9_1 = require
L10_1 = "JSON.lua"
L9_1 = L9_1(L10_1)
L10_1 = L8_1.activeCatalog
L10_1 = L10_1()
L11_1 = import
L12_1 = "LrColor"
L11_1 = L11_1(L12_1)
L12_1 = import
L13_1 = "LrPrefs"
L12_1 = L12_1(L13_1)
L12_1 = L12_1.prefsForPlugin
L12_1 = L12_1()
L13_1 = L0_1
L14_1 = "NLPRoll"
L13_1 = L13_1(L14_1)
L14_1 = _G
L14_1 = L14_1.dev
if L14_1 == true then
  L15_1 = L13_1
  L14_1 = L13_1.enable
  L16_1 = "logfile"
  L14_1(L15_1, L16_1)
else
  L15_1 = L13_1
  L14_1 = L13_1.disable
  L14_1(L15_1)
end
L14_1 = "minBlackMeanWhite"
L15_1 = nil
L16_1 = nil

function L17_1(A0_2)
  local L1_2, L2_2, L3_2
  L2_2 = A0_2 / 255
  if L2_2 <= 0.04045 then
    L1_2 = L2_2 / 12.92
  else
    L3_2 = L2_2 + 0.05213270142180095
    L1_2 = L3_2 ^ 2.4
  end
  L3_2 = L1_2 * 255
  return L3_2
end

function L18_1(A0_2)
  local L1_2, L2_2, L3_2
  L2_2 = A0_2 / 255
  if L2_2 <= 0.0031308 then
    L1_2 = 12.92 * L2_2
  else
    L3_2 = L2_2 ^ 0.4166666666666667
    L3_2 = 1.055 * L3_2
    L1_2 = L3_2 - 0.055
  end
  L3_2 = L1_2 * 255
  return L3_2
end

function L19_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A0_2]
  L2_2 = L2_2.aperture
  L3_2 = _G
  L3_2 = L3_2.activePhotosMeta
  L3_2 = L3_2[A0_2]
  L3_2 = L3_2.shutterSpeed
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A0_2]
  L4_2 = L4_2.iso
  if L2_2 and L3_2 and L4_2 then
    L5_2 = true
    L15_1 = L5_2
    L5_2 = L1_1
    L5_2 = L5_2.exposureValue
    L6_2 = L3_2
    L7_2 = L2_2
    L8_2 = L4_2
    L5_2 = L5_2(L6_2, L7_2, L8_2)
    L1_2 = L5_2
  end
  return L1_2
end

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L2_2 = {}
  L3_2 = L19_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L4_2 = true
    L15_1 = L4_2
    L2_2.ev = L3_2
  end
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A1_2]
  L4_2 = L4_2.previousLightroomSettings
  L5_2 = nil
  L6_2 = nil
  L7_2 = nil
  L8_2 = nil
  L9_2 = _G
  L9_2 = L9_2.activePhotosMeta
  L9_2 = L9_2[A1_2]
  L9_2 = L9_2.fileFormat
  if L9_2 ~= "RAW" then
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L9_2 = L9_2[A1_2]
    L9_2 = L9_2.fileFormat
    if L9_2 ~= "DNG" then
      goto lbl_32
    end
  end
  L5_2 = L4_2.Temperature
  L6_2 = L4_2.Tint
  L7_2 = "Temperature"
  L8_2 = "Tint"
  goto lbl_36
  ::lbl_32::
  L5_2 = L4_2.IncrementalTemperature
  L6_2 = L4_2.IncrementalTint
  L7_2 = "IncrementalTemperature"
  L8_2 = "IncrementalTint"
  ::lbl_36::
  if L5_2 and L5_2 then
    L9_2 = true
    L16_1 = L9_2
    L2_2.lrTemp = L5_2
    L2_2.lrTint = L6_2
  end
  if A1_2 then
    L9_2 = L1_1
    L9_2 = L9_2.copyTable
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L10_2 = L10_2[A1_2]
    L10_2 = L10_2.redChannel
    L9_2 = L9_2(L10_2)
    L2_2.redChannel = L9_2
    L9_2 = L1_1
    L9_2 = L9_2.copyTable
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L10_2 = L10_2[A1_2]
    L10_2 = L10_2.greenChannel
    L9_2 = L9_2(L10_2)
    L2_2.greenChannel = L9_2
    L9_2 = L1_1
    L9_2 = L9_2.copyTable
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L10_2 = L10_2[A1_2]
    L10_2 = L10_2.blueChannel
    L9_2 = L9_2(L10_2)
    L2_2.blueChannel = L9_2
    L9_2 = L1_1
    L9_2 = L9_2.copyTable
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L10_2 = L10_2[A1_2]
    L10_2 = L10_2.autoWB
    L9_2 = L9_2(L10_2)
    L2_2.autoWB = L9_2
  else
    L10_2 = A0_2
    L9_2 = A0_2.getPropertyForPlugin
    L11_2 = _PLUGIN
    L12_2 = "rgbOrigins"
    L9_2 = L9_2(L10_2, L11_2, L12_2)
    L11_2 = A0_2
    L10_2 = A0_2.getPropertyForPlugin
    L12_2 = _PLUGIN
    L13_2 = "autoWB"
    L10_2 = L10_2(L11_2, L12_2, L13_2)
    if L9_2 and L10_2 then
      L11_2 = L9_2.r
      L2_2.redChannel = L11_2
      L11_2 = L9_2.g
      L2_2.greenChannel = L11_2
      L11_2 = L9_2.b
      L2_2.blueChannel = L11_2
      L2_2.autoWB = L10_2
    else
      L11_2 = L4_1
      L11_2 = L11_2.message
      L12_2 = "ERROR"
      L13_2 = "Could not locate metadata necessary for roll analysis."
      L14_2 = "critical"
      L11_2(L12_2, L13_2, L14_2)
    end
  end
  return L2_2
end

function L21_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L2_2 = {}
  L3_2 = nil
  L4_2 = #A0_2
  if L4_2 == 1 then
    L4_2 = _G
    L3_2 = L4_2.activePhotoIndex
  end
  L4_2 = 1
  L5_2 = #A0_2
  L6_2 = 1
  for L7_2 = L4_2, L5_2, L6_2 do
    if A1_2 == true then
      L8_2 = L20_1
      L9_2 = A0_2[L7_2]
      L10_2 = L3_2 or L10_2
      if not L3_2 then
        L10_2 = L7_2
      end
      L8_2 = L8_2(L9_2, L10_2)
      L9_2 = table
      L9_2 = L9_2.insert
      L10_2 = L2_2
      L11_2 = L8_2
      L9_2(L10_2, L11_2)
    else
      L8_2 = L20_1
      L9_2 = A0_2[L7_2]
      L8_2 = L8_2(L9_2)
      L9_2 = table
      L9_2 = L9_2.insert
      L10_2 = L2_2
      L11_2 = L8_2
      L9_2(L10_2, L11_2)
    end
  end
  return L2_2
end

function L22_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L2_2 = A0_2.ev
  L3_2 = {}
  L4_2 = 0
  L5_2 = 1
  L6_2 = #L2_2
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L9_2 = math
    L9_2 = L9_2.abs
    L10_2 = L2_2[L8_2]
    L10_2 = L10_2 - A1_2
    L9_2 = L9_2(L10_2)
    if L4_2 < L9_2 then
      L4_2 = L9_2
    end
  end
  if 1 < L4_2 then
    L5_2 = "ERROR: Large Exposure Variance Detected"
    L6_2 = [[
Roll Analysis has detected unusually large differences in exposure metadata between frames. This can be caused by not properly setting exposure during digital capture, or by using 3rd party tools to overwrite EXIF exposure data of the scans.

You can choose to continue without adjusting for exposure, but the results may not be as accurate.]]
    L7_2 = "Continue (without exposure data)"
    L8_2 = "Cancel"
    L9_2 = L4_1
    L9_2 = L9_2.confirm
    L10_2 = L5_2
    L11_2 = L6_2
    L12_2 = L7_2
    L13_2 = L8_2
    L9_2 = L9_2(L10_2, L11_2, L12_2, L13_2)
    if L9_2 == "ok" then
      L10_2 = A0_2
      L11_2 = "exposure-off"
      return L10_2, L11_2
    else
      L10_2 = A0_2
      L11_2 = "cancel"
      return L10_2, L11_2
    end
  end
  L5_2 = #L2_2
  L6_2 = -1
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L9_2 = L2_2[L8_2]
    if L9_2 ~= A1_2 then
      L9_2 = L2_2[L8_2]
      L9_2 = L9_2 - A1_2
      L10_2 = A0_2.redChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.redChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.whitePointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.whitePointOrigin = L11_2
      L10_2 = A0_2.redChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.redChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.blackPointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.blackPointOrigin = L11_2
      L10_2 = A0_2.redChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.redChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.meanPoint
      L12_2 = L12_2 * 255
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L11_2 = L11_2 / 255
      L10_2.meanPoint = L11_2
      L10_2 = A0_2.greenChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.greenChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.whitePointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.whitePointOrigin = L11_2
      L10_2 = A0_2.greenChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.greenChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.blackPointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.blackPointOrigin = L11_2
      L10_2 = A0_2.greenChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.greenChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.meanPoint
      L12_2 = L12_2 * 255
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L11_2 = L11_2 / 255
      L10_2.meanPoint = L11_2
      L10_2 = A0_2.blueChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.blueChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.whitePointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.whitePointOrigin = L11_2
      L10_2 = A0_2.blueChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.blueChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.blackPointOrigin
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L10_2.blackPointOrigin = L11_2
      L10_2 = A0_2.blueChannel
      L10_2 = L10_2[L8_2]
      L11_2 = L1_1
      L11_2 = L11_2.adjustExposure
      L12_2 = A0_2.blueChannel
      L12_2 = L12_2[L8_2]
      L12_2 = L12_2.meanPoint
      L12_2 = L12_2 * 255
      L13_2 = L9_2
      L11_2 = L11_2(L12_2, L13_2)
      L11_2 = L11_2 / 255
      L10_2.meanPoint = L11_2
    end
  end
  return A0_2
end

function L23_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2
  L3_2 = 1
  L4_2 = {}
  L5_2 = {}
  L6_2 = L1_1
  L6_2 = L6_2.max
  L7_2 = A0_2.blackPointOrigin
  L6_2 = L6_2(L7_2)
  L7_2 = L1_1
  L7_2 = L7_2.max
  L8_2 = A1_2.blackPointOrigin
  L7_2 = L7_2(L8_2)
  L8_2 = L1_1
  L8_2 = L8_2.max
  L9_2 = A2_2.blackPointOrigin
  L8_2 = L8_2(L9_2)
  L9_2 = 1
  L10_2 = A0_2.whitePointOrigin
  L10_2 = #L10_2
  L11_2 = 1
  for L12_2 = L9_2, L10_2, L11_2 do
    L13_2 = L6_2 ^ L3_2
    L14_2 = A0_2.whitePointOrigin
    L14_2 = L14_2[L12_2]
    L14_2 = L14_2 ^ L3_2
    L13_2 = L13_2 - L14_2
    L14_2 = L7_2 ^ L3_2
    L15_2 = A1_2.whitePointOrigin
    L15_2 = L15_2[L12_2]
    L15_2 = L15_2 ^ L3_2
    L14_2 = L14_2 - L15_2
    L15_2 = L8_2 ^ L3_2
    L16_2 = A2_2.whitePointOrigin
    L16_2 = L16_2[L12_2]
    L16_2 = L16_2 ^ L3_2
    L15_2 = L15_2 - L16_2
    L16_2 = L14_2 / L13_2
    L17_2 = L15_2 / L13_2
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L4_2
    L20_2 = L16_2
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L5_2
    L20_2 = L17_2
    L18_2(L19_2, L20_2)
  end
  L9_2 = L1_1
  L9_2 = L9_2.standardDeviation
  L10_2 = L4_2
  L9_2, L10_2 = L9_2(L10_2)
  L11_2 = L1_1
  L11_2 = L11_2.standardDeviation
  L12_2 = L5_2
  L11_2, L12_2 = L11_2(L12_2)
  L13_2 = 2
  L14_2 = {}
  L15_2 = 1
  L16_2 = #L5_2
  L17_2 = 1
  for L18_2 = L15_2, L16_2, L17_2 do
    L19_2 = L13_2 * L9_2
    L20_2 = L13_2 * L11_2
    L21_2 = L5_2[L18_2]
    L22_2 = L12_2 + L20_2
    L21_2 = L21_2 > L22_2
    L22_2 = L4_2[L18_2]
    L23_2 = L10_2 + L19_2
    L22_2 = L22_2 > L23_2
    if L21_2 or L22_2 then
      L23_2 = table
      L23_2 = L23_2.insert
      L24_2 = L14_2
      L25_2 = L18_2
      L23_2(L24_2, L25_2)
    end
  end
  L15_2 = #L14_2
  L16_2 = 1
  L17_2 = -1
  for L18_2 = L15_2, L16_2, L17_2 do
    L19_2 = table
    L19_2 = L19_2.remove
    L20_2 = L4_2
    L21_2 = L14_2[L18_2]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.remove
    L20_2 = L5_2
    L21_2 = L14_2[L18_2]
    L19_2(L20_2, L21_2)
  end
  L15_2 = 1
  L16_2 = L1_1
  L16_2 = L16_2.mean
  L17_2 = L5_2
  L16_2 = L16_2(L17_2)
  L17_2 = L1_1
  L17_2 = L17_2.mean
  L18_2 = L4_2
  L17_2 = L17_2(L18_2)
  L18_2 = L16_2
  L19_2 = L17_2
  return L18_2, L19_2
end

function L24_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2
  L1_2 = {}
  L2_2 = {}
  L3_2 = {}
  L4_2 = {}
  L5_2 = {}
  L6_2 = {}
  L7_2 = {}
  L2_2.b = L7_2
  L2_2.g = L6_2
  L2_2.r = L5_2
  L5_2 = {}
  L6_2 = {}
  L7_2 = {}
  L3_2.b = L7_2
  L3_2.g = L6_2
  L3_2.r = L5_2
  L5_2 = {}
  L6_2 = {}
  L7_2 = {}
  L4_2.b = L7_2
  L4_2.g = L6_2
  L4_2.r = L5_2
  L5_2 = 1
  L6_2 = #A0_2
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L9_2 = A0_2[L8_2]
    L9_2 = L9_2.smartColorNeutral
    L10_2 = A0_2[L8_2]
    L10_2 = L10_2.smartColorWarming
    L11_2 = A0_2[L8_2]
    L11_2 = L11_2.smartColorCooling
    L12_2 = L1_1
    L12_2 = L12_2.formatHSV
    L13_2 = L9_2
    L12_2 = L12_2(L13_2)
    L13_2 = L1_1
    L13_2 = L13_2.formatHSV
    L14_2 = L10_2
    L13_2 = L13_2(L14_2)
    L14_2 = L1_1
    L14_2 = L14_2.formatHSV
    L15_2 = L11_2
    L14_2 = L14_2(L15_2)
    L15_2 = L1_1
    L15_2 = L15_2.HSVToRGB
    L16_2 = L12_2
    L17_2 = true
    L15_2 = L15_2(L16_2, L17_2)
    L16_2 = L1_1
    L16_2 = L16_2.HSVToRGB
    L17_2 = L13_2
    L18_2 = true
    L16_2 = L16_2(L17_2, L18_2)
    L17_2 = L1_1
    L17_2 = L17_2.HSVToRGB
    L18_2 = L14_2
    L19_2 = true
    L17_2 = L17_2(L18_2, L19_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L2_2.r
    L20_2 = L15_2[1]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L2_2.g
    L20_2 = L15_2[2]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L2_2.b
    L20_2 = L15_2[3]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L3_2.r
    L20_2 = L16_2[1]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L3_2.g
    L20_2 = L16_2[2]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L3_2.b
    L20_2 = L16_2[3]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L4_2.r
    L20_2 = L17_2[1]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L4_2.g
    L20_2 = L17_2[2]
    L18_2(L19_2, L20_2)
    L18_2 = table
    L18_2 = L18_2.insert
    L19_2 = L4_2.b
    L20_2 = L17_2[3]
    L18_2(L19_2, L20_2)
  end
  L5_2 = {}
  L6_2 = L1_1
  L6_2 = L6_2.rgbAvg
  L7_2 = L2_2.r
  L6_2 = L6_2(L7_2)
  L7_2 = L1_1
  L7_2 = L7_2.rgbAvg
  L8_2 = L2_2.g
  L7_2 = L7_2(L8_2)
  L8_2 = L1_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L2_2.b
  L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2 = L8_2(L9_2)
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
  L6_2 = {}
  L7_2 = L1_1
  L7_2 = L7_2.rgbAvg
  L8_2 = L3_2.r
  L7_2 = L7_2(L8_2)
  L8_2 = L1_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L3_2.g
  L8_2 = L8_2(L9_2)
  L9_2 = L1_1
  L9_2 = L9_2.rgbAvg
  L10_2 = L3_2.b
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2 = L9_2(L10_2)
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
  L7_2 = {}
  L8_2 = L1_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L4_2.r
  L8_2 = L8_2(L9_2)
  L9_2 = L1_1
  L9_2 = L9_2.rgbAvg
  L10_2 = L4_2.g
  L9_2 = L9_2(L10_2)
  L10_2 = L1_1
  L10_2 = L10_2.rgbAvg
  L11_2 = L4_2.b
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2 = L10_2(L11_2)
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
  L8_2 = L1_1
  L8_2 = L8_2.rgbToHsv
  L9_2 = L5_2[1]
  L10_2 = L5_2[2]
  L11_2 = L5_2[3]
  L8_2, L9_2, L10_2 = L8_2(L9_2, L10_2, L11_2)
  L11_2 = L1_1
  L11_2 = L11_2.rgbToHsv
  L12_2 = L6_2[1]
  L13_2 = L6_2[2]
  L14_2 = L6_2[3]
  L11_2, L12_2, L13_2 = L11_2(L12_2, L13_2, L14_2)
  L14_2 = L1_1
  L14_2 = L14_2.rgbToHsv
  L15_2 = L7_2[1]
  L16_2 = L7_2[2]
  L17_2 = L7_2[3]
  L14_2, L15_2, L16_2 = L14_2(L15_2, L16_2, L17_2)
  L17_2 = L1_1
  L17_2 = L17_2.unformatHSV
  L18_2 = {}
  L18_2.h = L8_2
  L18_2.s = L9_2
  L18_2.b = L10_2
  L17_2 = L17_2(L18_2)
  L18_2 = L1_1
  L18_2 = L18_2.unformatHSV
  L19_2 = {}
  L19_2.h = L11_2
  L19_2.s = L12_2
  L19_2.b = L13_2
  L18_2 = L18_2(L19_2)
  L19_2 = L1_1
  L19_2 = L19_2.unformatHSV
  L20_2 = {}
  L20_2.h = L14_2
  L20_2.s = L15_2
  L20_2.b = L16_2
  L19_2 = L19_2(L20_2)
  L20_2 = L17_2
  L21_2 = L18_2
  L22_2 = L19_2
  return L20_2, L21_2, L22_2
end

function L25_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = 1
  L3_2 = #A1_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L5_2]
    L7_2 = A0_2.rollName
    L6_2.rollName = L7_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L5_2]
    L7_2 = A0_2.ID
    L6_2.rollID = L7_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L5_2]
    L6_2.roll = A0_2
  end
end

function L26_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L10_1
  L2_2 = L1_2
  L1_2 = L1_2.withPrivateWriteAccessDo
  
  function L3_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
    L1_3 = 1
    L2_3 = A0_2
    L3_3 = 1
    for L4_3 = L1_3, L2_3, L3_3 do
      L5_3 = A0_2
      L5_3 = L5_3[L4_3]
      L6_3 = L5_3
      L5_3 = L5_3.setPropertyForPlugin
      L7_3 = _PLUGIN
      L8_3 = "roll"
      L9_3 = nil
      L5_3(L6_3, L7_3, L8_3, L9_3)
      L5_3 = A0_2
      L5_3 = L5_3[L4_3]
      L6_3 = L5_3
      L5_3 = L5_3.setPropertyForPlugin
      L7_3 = _PLUGIN
      L8_3 = "rollName"
      L9_3 = nil
      L5_3(L6_3, L7_3, L8_3, L9_3)
      L5_3 = A0_2
      L5_3 = L5_3[L4_3]
      L6_3 = L5_3
      L5_3 = L5_3.setPropertyForPlugin
      L7_3 = _PLUGIN
      L8_3 = "rollID"
      L9_3 = nil
      L5_3(L6_3, L7_3, L8_3, L9_3)
    end
  end
  
  L1_2 = L1_2(L2_2, L3_2)
end

function L27_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = "rollID_"
  L2_2 = A0_2.ID
  L1_2 = L1_2 .. L2_2
  L2_2 = _G
  L3_2 = _G
  L3_2 = L3_2.rolls
  if not L3_2 then
    L3_2 = {}
  end
  L2_2.rolls = L3_2
  L2_2 = _G
  L2_2 = L2_2.rolls
  L2_2[L1_2] = A0_2
  L2_2 = _G
  L2_2.rollAdded = true
end

function L28_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = createRollDropdown
  L1_2()
  L1_2 = properties
  L2_2 = A0_2.ID
  L1_2.rollAnalysis = L2_2
  L1_2 = {}
  L2_2 = 1
  L3_2 = _G
  L3_2 = L3_2.activePhotos
  L3_2 = #L3_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = _G
    L6_2 = L6_2.activePhotos
    L6_2 = L6_2[L5_2]
    L7_2 = {}
    L8_2 = _G
    L8_2 = L8_2.activePhotosMeta
    L8_2 = L8_2[L5_2]
    L8_2 = L8_2.redChannel
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L9_2 = L9_2[L5_2]
    L9_2 = L9_2.greenChannel
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L10_2 = L10_2[L5_2]
    L10_2 = L10_2.blueChannel
    L7_2[1] = L8_2
    L7_2[2] = L9_2
    L7_2[3] = L10_2
    L8_2 = _G
    L8_2 = L8_2.activePhotosMeta
    L8_2 = L8_2[L5_2]
    L8_2 = L8_2.settings
    L9_2 = A0_2.ID
    L8_2.rollAnalysis = L9_2
    L8_2 = table
    L8_2 = L8_2.insert
    L9_2 = L1_2
    L10_2 = {}
    L10_2.photo = L6_2
    L10_2.rgbInput = L7_2
    L11_2 = _G
    L11_2 = L11_2.activePhotosMeta
    L11_2 = L11_2[L5_2]
    L11_2 = L11_2.settings
    L10_2.properties = L11_2
    L8_2(L9_2, L10_2)
  end
  L2_2 = NLPViewController
  L2_2 = L2_2.updateAllAdjustments
  L3_2 = L1_2
  L4_2 = uiSettings
  L2_2(L3_2, L4_2)
end

function L29_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = {}
  L1_2.separator = true
  L2_2 = {}
  L3_2 = next
  L4_2 = _G
  L4_2 = L4_2.rolls
  L3_2 = L3_2(L4_2)
  if L3_2 ~= nil then
    L3_2 = next
    L4_2 = _G
    L4_2 = L4_2.rolls
    L4_2 = L4_2.groups
    L3_2 = L3_2(L4_2)
    if L3_2 ~= nil then
      L3_2 = {}
      L4_2 = pairs
      L5_2 = _G
      L5_2 = L5_2.rolls
      L5_2 = L5_2.groups
      L4_2, L5_2, L6_2 = L4_2(L5_2)
      for L7_2 in L4_2, L5_2, L6_2 do
        L8_2 = table
        L8_2 = L8_2.insert
        L9_2 = L3_2
        L10_2 = L7_2
        L8_2(L9_2, L10_2)
      end
      L4_2 = table
      L4_2 = L4_2.sort
      L5_2 = L3_2
      L4_2(L5_2)
      L4_2 = 1
      L5_2 = #L3_2
      L6_2 = 1
      for L7_2 = L4_2, L5_2, L6_2 do
        L8_2 = table
        L8_2 = L8_2.insert
        L9_2 = L2_2
        L10_2 = {}
        L11_2 = L3_2[L7_2]
        L10_2.title = L11_2
        L10_2.value = L7_2
        L8_2(L9_2, L10_2)
      end
  end
  else
    L3_2 = table
    L3_2 = L3_2.insert
    L4_2 = L2_2
    L5_2 = {}
    L5_2.title = "User"
    L5_2.value = 1
    L3_2(L4_2, L5_2)
  end
  if A0_2 ~= nil then
    L3_2 = table
    L3_2 = L3_2.insert
    L4_2 = L2_2
    L5_2 = A0_2
    L3_2(L4_2, L5_2)
  end
  L3_2 = table
  L3_2 = L3_2.insert
  L4_2 = L2_2
  L5_2 = L1_2
  L3_2(L4_2, L5_2)
  L3_2 = table
  L3_2 = L3_2.insert
  L4_2 = L2_2
  L5_2 = {}
  L5_2.title = "New Group..."
  L5_2.value = "new"
  L3_2(L4_2, L5_2)
  return L2_2
end

function L30_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L3_2 = A0_2
  L2_2 = A0_2.column
  L4_2 = {}
  L6_2 = A0_2
  L5_2 = A0_2.control_spacing
  L5_2 = L5_2(L6_2)
  L4_2.spacing = L5_2
  L6_2 = A0_2
  L5_2 = A0_2.row
  L7_2 = {}
  L7_2.fill_horizontal = 1
  L9_2 = A0_2
  L8_2 = A0_2.static_text
  L10_2 = {}
  L10_2.title = "Name: "
  L10_2.alignment = "right"
  L11_2 = L5_1
  L11_2 = L11_2.share
  L12_2 = "roll_label_width"
  L11_2 = L11_2(L12_2)
  L10_2.width = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.edit_field
  L11_2 = {}
  L11_2.fill_horizontal = 1
  L12_2 = L5_1
  L12_2 = L12_2.bind
  L13_2 = "rollName"
  L12_2 = L12_2(L13_2)
  L11_2.value = L12_2
  L11_2.alignment = "left"
  L11_2.immediate = true
  
  function L12_2(A0_3, A1_3)
    local L2_3, L3_3
    L2_3 = string
    L2_3 = L2_3.len
    L3_3 = A1_3
    L2_3 = L2_3(L3_3)
    if 0 < L2_3 then
      L2_3 = A1_2
      L2_3.validName = true
      L2_3 = true
      L3_3 = A1_3
      return L2_3, L3_3
    else
      L2_3 = A1_2
      L2_3.validName = false
      L2_3 = false
      L3_3 = A1_3
      return L2_3, L3_3
    end
  end
  
  L11_2.validate = L12_2
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2 = L9_2(L10_2, L11_2)
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
  L5_2 = L5_2(L6_2, L7_2)
  L7_2 = A0_2
  L6_2 = A0_2.row
  L8_2 = {}
  L8_2.fill_horizontal = 1
  L10_2 = A0_2
  L9_2 = A0_2.static_text
  L11_2 = {}
  L11_2.title = "Group: "
  L11_2.alignment = "right"
  L12_2 = L5_1
  L12_2 = L12_2.share
  L13_2 = "roll_label_width"
  L12_2 = L12_2(L13_2)
  L11_2.width = L12_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.popup_menu
  L12_2 = {}
  L12_2.alignment = "left"
  L13_2 = L5_1
  L13_2 = L13_2.bind
  L14_2 = "group"
  L13_2 = L13_2(L14_2)
  L12_2.value = L13_2
  L12_2.fill_horizontal = 1
  L13_2 = L5_1
  L13_2 = L13_2.bind
  L14_2 = "groupDropdown"
  L13_2 = L13_2(L14_2)
  L12_2.items = L13_2
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2 = L10_2(L11_2, L12_2)
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
  L6_2 = L6_2(L7_2, L8_2)
  L8_2 = A0_2
  L7_2 = A0_2.separator
  L9_2 = {}
  L9_2.fill_horizontal = 1
  L7_2 = L7_2(L8_2, L9_2)
  L9_2 = A0_2
  L8_2 = A0_2.row
  L10_2 = {}
  L10_2.fill_horizontal = 1
  L12_2 = A0_2
  L11_2 = A0_2.static_text
  L13_2 = {}
  L13_2.title = "Source: "
  L13_2.alignment = "right"
  L14_2 = L5_1
  L14_2 = L14_2.share
  L15_2 = "roll_label_width"
  L14_2 = L14_2(L15_2)
  L13_2.width = L14_2
  L11_2 = L11_2(L12_2, L13_2)
  L13_2 = A0_2
  L12_2 = A0_2.popup_menu
  L14_2 = {}
  L14_2.alignment = "left"
  L14_2.fill_horizontal = 1
  L15_2 = L5_1
  L15_2 = L15_2.bind
  L16_2 = "rollSource"
  L15_2 = L15_2(L16_2)
  L14_2.value = L15_2
  L15_2 = {}
  L16_2 = {}
  L16_2.title = "All Selected Photos"
  L16_2.value = "selection"
  L17_2 = {}
  L17_2.title = "Single Frame"
  L17_2.value = "single"
  L15_2[1] = L16_2
  L15_2[2] = L17_2
  L14_2.items = L15_2
  L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2 = L12_2(L13_2, L14_2)
  L10_2[1] = L11_2
  L10_2[2] = L12_2
  L10_2[3] = L13_2
  L10_2[4] = L14_2
  L10_2[5] = L15_2
  L10_2[6] = L16_2
  L10_2[7] = L17_2
  L10_2[8] = L18_2
  L10_2[9] = L19_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.row
  L11_2 = {}
  L13_2 = A0_2
  L12_2 = A0_2.spacer
  L14_2 = {}
  L15_2 = L5_1
  L15_2 = L15_2.share
  L16_2 = "roll_label_width"
  L15_2 = L15_2(L16_2)
  L14_2.width = L15_2
  L12_2 = L12_2(L13_2, L14_2)
  L14_2 = A0_2
  L13_2 = A0_2.static_text
  L15_2 = {}
  L15_2.width_in_chars = 25
  L15_2.selectable = true
  L15_2.height_in_lines = 2
  L15_2.size = "small"
  L16_2 = L11_1
  L17_2 = 0.2
  L18_2 = 0.2
  L19_2 = 0.2
  L16_2 = L16_2(L17_2, L18_2, L19_2)
  L15_2.text_color = L16_2
  L16_2 = L5_1
  L16_2 = L16_2.bind
  L17_2 = {}
  L17_2.key = "rollSource"
  
  function L18_2(A0_3, A1_3)
    local L2_3
    if A1_3 then
      if A0_3 == "selection" then
        L2_3 = "Create a new roll analysis based on all of the images you currently have selected."
        return L2_3
      elseif A0_3 == "folder" then
        L2_3 = "Create a new roll based on all converted images in this folder."
        return L2_3
      elseif A0_3 == "single" then
        L2_3 = "Create a calibration based on this single image."
        return L2_3
      else
        L2_3 = ""
        return L2_3
      end
    end
    L2_3 = L6_1
    L2_3 = L2_3.kUnsupportedDirection
    return L2_3
  end
  
  L17_2.transform = L18_2
  L16_2 = L16_2(L17_2)
  L15_2.title = L16_2
  L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2 = L13_2(L14_2, L15_2)
  L11_2[1] = L12_2
  L11_2[2] = L13_2
  L11_2[3] = L14_2
  L11_2[4] = L15_2
  L11_2[5] = L16_2
  L11_2[6] = L17_2
  L11_2[7] = L18_2
  L11_2[8] = L19_2
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2 = L9_2(L10_2, L11_2)
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
  L2_2 = L2_2(L3_2, L4_2)
  L4_2 = A0_2
  L3_2 = A0_2.column
  L5_2 = {}
  L5_2.bind_to_object = A1_2
  L6_2 = L2_2
  L5_2[1] = L6_2
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

function L31_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L3_2 = A0_2
  L2_2 = A0_2.row
  L4_2 = {}
  L4_2.bind_to_object = A1_2
  L6_2 = A0_2
  L5_2 = A0_2.static_text
  L7_2 = {}
  L7_2.title = "Group Name: "
  L5_2 = L5_2(L6_2, L7_2)
  L7_2 = A0_2
  L6_2 = A0_2.edit_field
  L8_2 = {}
  L8_2.fill_horizontal = 1
  L8_2.immediate = true
  L9_2 = L5_1
  L9_2 = L9_2.bind
  L10_2 = "newGroupName"
  L9_2 = L9_2(L10_2)
  L8_2.value = L9_2
  L8_2.alignment = "left"
  
  function L9_2(A0_3, A1_3)
    local L2_3, L3_3
    L2_3 = string
    L2_3 = L2_3.len
    L3_3 = A1_3
    L2_3 = L2_3(L3_3)
    if 0 < L2_3 then
      L2_3 = A1_2
      L2_3.validGroupName = true
      L2_3 = true
      L3_3 = A1_3
      return L2_3, L3_3
    else
      L2_3 = A1_2
      L2_3.validGroupName = false
      L2_3 = false
      L3_3 = A1_3
      return L2_3, L3_3
    end
  end
  
  L8_2.validate = L9_2
  L6_2, L7_2, L8_2, L9_2, L10_2 = L6_2(L7_2, L8_2)
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L4_2[4] = L8_2
  L4_2[5] = L9_2
  L4_2[6] = L10_2
  L2_2 = L2_2(L3_2, L4_2)
  return L2_2
end

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = {}
  L3_2 = _G
  L3_2 = L3_2.activePhotosMeta
  L3_2 = L3_2[A1_2]
  L3_2 = L3_2.redChannel
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A1_2]
  L4_2 = L4_2.greenChannel
  L5_2 = _G
  L5_2 = L5_2.activePhotosMeta
  L5_2 = L5_2[A1_2]
  L5_2 = L5_2.blueChannel
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  return L2_2
end

getIndividualPhotoOrigins = L32_1
L32_1 = {}
NLPRolls = L32_1

function L32_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2)
  local L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L6_2 = {}
  L7_2 = A0_2[A3_2]
  L7_2 = L7_2.blackPointOrigin
  L8_2 = A0_2[A3_2]
  L8_2 = L8_2.whitePointOrigin
  L7_2 = L7_2 - L8_2
  L8_2 = L7_2 - A2_2
  L8_2 = L8_2 / L7_2
  L9_2 = A0_2[A5_2]
  L9_2 = L9_2.blackPointOrigin
  L10_2 = A0_2[A5_2]
  L10_2 = L10_2.whitePointOrigin
  L9_2 = L9_2 - L10_2
  L10_2 = L9_2 + A4_2
  L10_2 = L10_2 / L9_2
  L11_2 = 1
  L12_2 = #A0_2
  L13_2 = 1
  for L14_2 = L11_2, L12_2, L13_2 do
    L15_2 = A0_2[L14_2]
    L15_2 = L15_2.whitePointOrigin
    L16_2 = A0_2[L14_2]
    L16_2 = L16_2.blackPointOrigin
    L17_2 = L16_2 - L15_2
    L18_2 = {}
    L6_2[L14_2] = L18_2
    L18_2 = L6_2[L14_2]
    L19_2 = L17_2 * L8_2
    L19_2 = L16_2 - L19_2
    L18_2.whitePointOrigin = L19_2
    L18_2 = L6_2[L14_2]
    L19_2 = L17_2 * L10_2
    L19_2 = L15_2 + L19_2
    L18_2.blackPointOrigin = L19_2
  end
  L11_2 = {}
  L12_2 = L6_2[1]
  L12_2 = L12_2.blackPointOrigin
  L11_2.blackPointOrigin = L12_2
  L12_2 = L6_2[1]
  L12_2 = L12_2.whitePointOrigin
  L11_2.whitePointOrigin = L12_2
  L12_2 = A1_2.rMean
  L11_2.meanPoint = L12_2
  L11_2.settingName = "ToneCurvePV2012Red"
  L12_2 = {}
  L13_2 = L6_2[2]
  L13_2 = L13_2.blackPointOrigin
  L12_2.blackPointOrigin = L13_2
  L13_2 = L6_2[2]
  L13_2 = L13_2.whitePointOrigin
  L12_2.whitePointOrigin = L13_2
  L13_2 = A1_2.gMean
  L12_2.meanPoint = L13_2
  L12_2.settingName = "ToneCurvePV2012Green"
  L13_2 = {}
  L14_2 = L6_2[3]
  L14_2 = L14_2.blackPointOrigin
  L13_2.blackPointOrigin = L14_2
  L14_2 = L6_2[3]
  L14_2 = L14_2.whitePointOrigin
  L13_2.whitePointOrigin = L14_2
  L14_2 = A1_2.bMean
  L13_2.meanPoint = L14_2
  L13_2.settingName = "ToneCurvePV2012Blue"
  L14_2 = {}
  L15_2 = L11_2
  L16_2 = L12_2
  L17_2 = L13_2
  L14_2[1] = L15_2
  L14_2[2] = L16_2
  L14_2[3] = L17_2
  return L14_2
end

function L33_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2
  L2_2 = 2
  L3_2 = A0_2.greenMultiplier
  L4_2 = A0_2.blueMultiplier
  L5_2 = A0_2.rBlack
  L6_2 = A0_2.gBlack
  L7_2 = A0_2.bBlack
  L8_2 = nil
  L9_2 = _G
  L9_2 = L9_2.activePhotosMeta
  L8_2 = L9_2[A1_2]
  L9_2 = nil
  L10_2 = nil
  L11_2 = nil
  L12_2 = L8_2.redChannel
  L12_2 = L12_2.whitePointOrigin
  L12_2 = L5_2 - L12_2
  L9_2 = L12_2 ^ L2_2
  L10_2 = L9_2 * L3_2
  L11_2 = L9_2 * L4_2
  L12_2 = L8_2.redChannel
  L12_2 = L12_2.whitePointOrigin
  L13_2 = 1 / L2_2
  L13_2 = L10_2 ^ L13_2
  L13_2 = L6_2 - L13_2
  L14_2 = 1 / L2_2
  L14_2 = L11_2 ^ L14_2
  L14_2 = L7_2 - L14_2
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = L8_2.greenChannel
  L18_2 = L18_2.whitePointOrigin
  L18_2 = L13_2 - L18_2
  L19_2 = L8_2.blueChannel
  L19_2 = L19_2.whitePointOrigin
  L19_2 = L14_2 - L19_2
  if 0 < L18_2 or 0 < L19_2 then
    if L18_2 > L19_2 then
      L20_2 = L8_2.greenChannel
      L20_2 = L20_2.whitePointOrigin
      L20_2 = L6_2 - L20_2
      L17_2 = L20_2 ^ L2_2
      L15_2 = L17_2 / L3_2
      L16_2 = L15_2 * L4_2
    else
      L20_2 = L8_2.blueChannel
      L20_2 = L20_2.whitePointOrigin
      L20_2 = L7_2 - L20_2
      L16_2 = L20_2 ^ L2_2
      L15_2 = L16_2 / L4_2
      L17_2 = L15_2 * L3_2
    end
    L20_2 = 1 / L2_2
    L20_2 = L15_2 ^ L20_2
    L12_2 = L5_2 - L20_2
    L20_2 = 1 / L2_2
    L20_2 = L17_2 ^ L20_2
    L13_2 = L6_2 - L20_2
    L20_2 = 1 / L2_2
    L20_2 = L16_2 ^ L20_2
    L14_2 = L7_2 - L20_2
  end
  L20_2 = L8_2.redChannel
  L20_2 = L20_2.blackPointOrigin
  L20_2 = L5_2 - L20_2
  L21_2 = L8_2.greenChannel
  L21_2 = L21_2.blackPointOrigin
  L21_2 = L6_2 - L21_2
  L22_2 = L8_2.blueChannel
  L22_2 = L22_2.blackPointOrigin
  L22_2 = L7_2 - L22_2
  L23_2 = nil
  L24_2 = nil
  L25_2 = nil
  L26_2 = L20_2 + L21_2
  L26_2 = L26_2 + L22_2
  L26_2 = L26_2 / 3
  if 2 < L26_2 then
    L27_2 = L26_2 / -2.2
    densityCompensation = L27_2
    L27_2 = densityCompensation
    if L27_2 < -30 then
      L27_2 = -30
      densityCompensation = L27_2
    end
  end
  if L20_2 < 0 then
    L27_2 = L8_2.redChannel
    L23_2 = L27_2.blackPointOrigin
  else
    L23_2 = L5_2
  end
  if L21_2 < 0 then
    L27_2 = L8_2.greenChannel
    L24_2 = L27_2.blackPointOrigin
  else
    L24_2 = L6_2
  end
  if L22_2 < 0 then
    L27_2 = L8_2.blueChannel
    L25_2 = L27_2.blackPointOrigin
  else
    L25_2 = L7_2
  end
  L27_2 = {}
  L27_2.blackPointOrigin = L23_2
  L27_2.whitePointOrigin = L12_2
  L28_2 = A0_2.rMean
  L27_2.meanPoint = L28_2
  L27_2.settingName = "ToneCurvePV2012Red"
  newRedChannel = L27_2
  L27_2 = {}
  L27_2.blackPointOrigin = L24_2
  L27_2.whitePointOrigin = L13_2
  L28_2 = A0_2.gMean
  L27_2.meanPoint = L28_2
  L27_2.settingName = "ToneCurvePV2012Green"
  newGreenChannel = L27_2
  L27_2 = {}
  L27_2.blackPointOrigin = L25_2
  L27_2.whitePointOrigin = L14_2
  L28_2 = A0_2.bMean
  L27_2.meanPoint = L28_2
  L27_2.settingName = "ToneCurvePV2012Blue"
  newBlueChannel = L27_2
  L27_2 = {}
  L28_2 = newRedChannel
  L29_2 = newGreenChannel
  L30_2 = newBlueChannel
  L27_2[1] = L28_2
  L27_2[2] = L29_2
  L27_2[3] = L30_2
  return L27_2
end

function L34_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2
  L2_2 = 1
  L3_2 = nil
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L3_2 = L4_2[A1_2]
  L4_2 = A0_2.rBlack
  L4_2 = L4_2 ^ L2_2
  L5_2 = L3_2.redChannel
  L5_2 = L5_2.whitePointOrigin
  L5_2 = L5_2 ^ L2_2
  L4_2 = L4_2 - L5_2
  L5_2 = A0_2.gBlack
  L5_2 = L5_2 ^ L2_2
  L6_2 = L3_2.greenChannel
  L6_2 = L6_2.whitePointOrigin
  L6_2 = L6_2 ^ L2_2
  L5_2 = L5_2 - L6_2
  L6_2 = A0_2.bBlack
  L6_2 = L6_2 ^ L2_2
  L7_2 = L3_2.blueChannel
  L7_2 = L7_2.whitePointOrigin
  L7_2 = L7_2 ^ L2_2
  L6_2 = L6_2 - L7_2
  L7_2 = L5_2 / L4_2
  L8_2 = L6_2 / L4_2
  L9_2 = A0_2.greenMultiplier
  L10_2 = A0_2.blueMultiplier
  L11_2 = 1 / L9_2
  L12_2 = L10_2 / L9_2
  L13_2 = 1 / L10_2
  L14_2 = L9_2 / L10_2
  L15_2 = A0_2.rBlack
  L16_2 = A0_2.gBlack
  L17_2 = A0_2.bBlack
  L18_2 = nil
  L19_2 = nil
  L20_2 = nil
  L21_2 = nil
  L22_2 = nil
  L23_2 = nil
  L24_2 = nil
  L25_2 = nil
  L26_2 = nil
  L27_2 = nil
  L28_2 = nil
  L29_2 = nil
  L30_2 = L9_2 - L7_2
  L31_2 = L10_2 - L8_2
  if L30_2 < 0 and L30_2 < L31_2 then
    L32_2 = L3_2.greenChannel
    L32_2 = L32_2.whitePointOrigin
    L20_2 = L32_2 ^ L2_2
    L32_2 = L16_2 ^ L2_2
    L32_2 = L32_2 - L20_2
    L33_2 = L15_2 ^ L2_2
    L34_2 = L32_2 * L11_2
    L18_2 = L33_2 - L34_2
    L33_2 = L17_2 ^ L2_2
    L34_2 = L32_2 * L12_2
    L22_2 = L33_2 - L34_2
    L33_2 = L3_2.greenChannel
    L33_2 = L33_2.blackPointOrigin
    L21_2 = L33_2 ^ L2_2
    L33_2 = L21_2 - L20_2
    L34_2 = L33_2 * L11_2
    L19_2 = L18_2 + L34_2
    L34_2 = L33_2 * L12_2
    L23_2 = L22_2 + L34_2
  elseif L31_2 < 0 and L30_2 > L31_2 then
    L32_2 = L3_2.blueChannel
    L32_2 = L32_2.whitePointOrigin
    L22_2 = L32_2 ^ L2_2
    L32_2 = L17_2 ^ L2_2
    L32_2 = L32_2 - L22_2
    L33_2 = L15_2 ^ L2_2
    L34_2 = L32_2 * L13_2
    L18_2 = L33_2 - L34_2
    L33_2 = L16_2 ^ L2_2
    L34_2 = L32_2 * L14_2
    L20_2 = L33_2 - L34_2
    L33_2 = L3_2.blueChannel
    L33_2 = L33_2.blackPointOrigin
    L23_2 = L33_2 ^ L2_2
    L33_2 = L23_2 - L22_2
    L34_2 = L33_2 * L13_2
    L19_2 = L18_2 + L34_2
    L34_2 = L33_2 * L14_2
    L21_2 = L20_2 + L34_2
  else
    L32_2 = L3_2.redChannel
    L32_2 = L32_2.whitePointOrigin
    L18_2 = L32_2 ^ L2_2
    L32_2 = L15_2 ^ L2_2
    L32_2 = L32_2 - L18_2
    L33_2 = L16_2 ^ L2_2
    L34_2 = L32_2 * L9_2
    L20_2 = L33_2 - L34_2
    L33_2 = L17_2 ^ L2_2
    L34_2 = L32_2 * L10_2
    L22_2 = L33_2 - L34_2
    L33_2 = L3_2.redChannel
    L33_2 = L33_2.blackPointOrigin
    L19_2 = L33_2 ^ L2_2
    L33_2 = L19_2 - L18_2
    L34_2 = L33_2 * L9_2
    L21_2 = L20_2 + L34_2
    L34_2 = L33_2 * L10_2
    L23_2 = L22_2 + L34_2
  end
  L32_2 = 1 / L2_2
  L24_2 = L18_2 ^ L32_2
  L32_2 = 1 / L2_2
  L26_2 = L20_2 ^ L32_2
  L32_2 = 1 / L2_2
  L28_2 = L22_2 ^ L32_2
  L32_2 = 1 / L2_2
  L25_2 = L19_2 ^ L32_2
  L32_2 = 1 / L2_2
  L27_2 = L21_2 ^ L32_2
  L32_2 = 1 / L2_2
  L29_2 = L23_2 ^ L32_2
  L32_2 = {}
  L32_2.blackPointOrigin = L25_2
  L32_2.whitePointOrigin = L24_2
  L33_2 = A0_2.rMean
  L32_2.meanPoint = L33_2
  L32_2.settingName = "ToneCurvePV2012Red"
  newRedChannel = L32_2
  L32_2 = {}
  L32_2.blackPointOrigin = L27_2
  L32_2.whitePointOrigin = L26_2
  L33_2 = A0_2.gMean
  L32_2.meanPoint = L33_2
  L32_2.settingName = "ToneCurvePV2012Green"
  newGreenChannel = L32_2
  L32_2 = {}
  L32_2.blackPointOrigin = L29_2
  L32_2.whitePointOrigin = L28_2
  L33_2 = A0_2.bMean
  L32_2.meanPoint = L33_2
  L32_2.settingName = "ToneCurvePV2012Blue"
  newBlueChannel = L32_2
  L32_2 = {}
  L33_2 = newRedChannel
  L34_2 = newGreenChannel
  L35_2 = newBlueChannel
  L32_2[1] = L33_2
  L32_2[2] = L34_2
  L32_2[3] = L35_2
  return L32_2
end

L35_1 = NLPRolls

function L36_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2, L41_2, L42_2, L43_2, L44_2, L45_2, L46_2, L47_2, L48_2, L49_2, L50_2, L51_2, L52_2
  L2_2 = RollCache
  if not L2_2 then
    L2_2 = {}
  end
  RollCache = L2_2
  L2_2 = RollCache
  L3_2 = RollCache
  L3_2 = L3_2[A1_2]
  if not L3_2 then
    L3_2 = {}
  end
  L2_2[A1_2] = L3_2
  L2_2 = A0_2.rollAnalysis
  L3_2 = "_"
  L4_2 = A0_2.rollMode
  L2_2 = L2_2 .. L3_2 .. L4_2
  L3_2 = {}
  L4_2 = 0
  L5_2 = 0
  L6_2 = 0
  L7_2 = A0_2.rollAnalysis
  if L7_2 ~= "individual" then
    L7_2 = A0_2.rollAnalysis
    if L7_2 ~= nil then
      L7_2 = _G
      L7_2 = L7_2.rolls
      if L7_2 ~= nil then
        goto lbl_46
      end
      L7_2 = _G
      L7_2 = L7_2.activePhotosMeta
      L7_2 = L7_2[A1_2]
      L7_2 = L7_2.roll
      if L7_2 ~= nil then
        goto lbl_46
      end
    end
  end
  L7_2 = getIndividualPhotoOrigins
  L8_2 = A0_2
  L9_2 = A1_2
  L7_2 = L7_2(L8_2, L9_2)
  L3_2 = L7_2
  L7_2 = L3_2
  L8_2 = L4_2
  do return L7_2, L8_2 end
  goto lbl_470
  ::lbl_46::
  L7_2 = "rollID_"
  L8_2 = A0_2.rollAnalysis
  L7_2 = L7_2 .. L8_2
  L8_2 = _G
  L8_2 = L8_2.rolls
  L8_2 = L8_2[L7_2]
  if not L8_2 then
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L9_2 = L9_2[A1_2]
    L9_2 = L9_2.roll
    if L9_2 ~= nil then
      L9_2 = _G
      L9_2 = L9_2.activePhotosMeta
      L9_2 = L9_2[A1_2]
      L9_2 = L9_2.roll
      L9_2 = L9_2.ID
      L10_2 = A0_2.rollAnalysis
      if L9_2 == L10_2 then
        L9_2 = _G
        L9_2 = L9_2.activePhotosMeta
        L9_2 = L9_2[A1_2]
        L8_2 = L9_2.roll
    end
    else
      L9_2 = getIndividualPhotoOrigins
      L10_2 = A0_2
      L11_2 = A1_2
      L9_2 = L9_2(L10_2, L11_2)
      L3_2 = L9_2
      L9_2 = L3_2
      L10_2 = L4_2
      return L9_2, L10_2
    end
  end
  L9_2 = RollCache
  L9_2 = L9_2[A1_2]
  L9_2 = L9_2[L2_2]
  if L9_2 ~= nil then
    L9_2 = RollCache
    L9_2 = L9_2[A1_2]
    L9_2 = L9_2[L2_2]
    L9_2 = L9_2.photoOrigins
    L10_2 = RollCache
    L10_2 = L10_2[A1_2]
    L10_2 = L10_2[L2_2]
    L10_2 = L10_2.densityCompensation
    L11_2 = RollCache
    L11_2 = L11_2[A1_2]
    L11_2 = L11_2[L2_2]
    L11_2 = L11_2.rollWhiteClip
    L12_2 = RollCache
    L12_2 = L12_2[A1_2]
    L12_2 = L12_2[L2_2]
    L12_2 = L12_2.rollBlackClip
    return L9_2, L10_2, L11_2, L12_2
  end
  L9_2 = L1_1
  L9_2 = L9_2.copyTable
  L10_2 = L8_2
  L9_2 = L9_2(L10_2)
  L10_2 = nil
  L11_2 = nil
  L12_2 = L9_2.processVersion
  if L12_2 ~= nil then
    L10_2 = L9_2.processVersion
  else
    L10_2 = nil
  end
  L12_2 = L9_2.commonEV
  L13_2 = L19_1
  L14_2 = A1_2
  L13_2 = L13_2(L14_2)
  if L13_2 and L12_2 and L13_2 ~= L12_2 and L12_2 ~= 0 then
    L14_2 = L12_2 - L13_2
    photoEVDiff = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.rBlack
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.rBlack = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.rWhite
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.rWhite = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.gBlack
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.gBlack = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.gWhite
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.gWhite = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.bBlack
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.bBlack = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.adjustExposure
    L15_2 = L9_2.bWhite
    L16_2 = photoEVDiff
    L14_2 = L14_2(L15_2, L16_2)
    L9_2.bWhite = L14_2
  end
  L14_2 = A0_2.rollMode
  if L14_2 ~= "cinema" then
    L14_2 = A0_2.rollMode
    if L14_2 ~= "drumscan" then
      goto lbl_201
    end
  end
  L14_2 = {}
  L15_2 = L9_2.rBlack
  L14_2.blackPointOrigin = L15_2
  L15_2 = L9_2.rWhite
  L14_2.whitePointOrigin = L15_2
  L15_2 = L9_2.rMean
  L14_2.meanPoint = L15_2
  L14_2.settingName = "ToneCurvePV2012Red"
  L15_2 = {}
  L16_2 = L9_2.gBlack
  L15_2.blackPointOrigin = L16_2
  L16_2 = L9_2.gWhite
  L15_2.whitePointOrigin = L16_2
  L16_2 = L9_2.gMean
  L15_2.meanPoint = L16_2
  L15_2.settingName = "ToneCurvePV2012Green"
  L16_2 = {}
  L17_2 = L9_2.bBlack
  L16_2.blackPointOrigin = L17_2
  L17_2 = L9_2.bWhite
  L16_2.whitePointOrigin = L17_2
  L17_2 = L9_2.bMean
  L16_2.meanPoint = L17_2
  L16_2.settingName = "ToneCurvePV2012Blue"
  L17_2 = {}
  L18_2 = L14_2
  L19_2 = L15_2
  L20_2 = L16_2
  L17_2[1] = L18_2
  L17_2[2] = L19_2
  L17_2[3] = L20_2
  L3_2 = L17_2
  goto lbl_470
  ::lbl_201::
  L14_2 = A0_2.rollMode
  if L14_2 == "optimized" then
    L14_2 = 0.45454545454545453
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[A1_2]
    originalOrigins = L15_2
    L15_2 = L9_2.rBlack
    L15_2 = L15_2 ^ L14_2
    L16_2 = originalOrigins
    L16_2 = L16_2.redChannel
    L16_2 = L16_2.whitePointOrigin
    L16_2 = L16_2 ^ L14_2
    L15_2 = L15_2 - L16_2
    L16_2 = L9_2.gBlack
    L16_2 = L16_2 ^ L14_2
    L17_2 = originalOrigins
    L17_2 = L17_2.greenChannel
    L17_2 = L17_2.whitePointOrigin
    L17_2 = L17_2 ^ L14_2
    L16_2 = L16_2 - L17_2
    L17_2 = L9_2.bBlack
    L17_2 = L17_2 ^ L14_2
    L18_2 = originalOrigins
    L18_2 = L18_2.blueChannel
    L18_2 = L18_2.whitePointOrigin
    L18_2 = L18_2 ^ L14_2
    L17_2 = L17_2 - L18_2
    L18_2 = L16_2 / L15_2
    L19_2 = L17_2 / L15_2
    L20_2 = L9_2.gBlack
    L20_2 = L20_2 ^ L14_2
    L21_2 = L9_2.gWhite
    L21_2 = L21_2 ^ L14_2
    L20_2 = L20_2 - L21_2
    L21_2 = L9_2.rBlack
    L21_2 = L21_2 ^ L14_2
    L22_2 = L9_2.rWhite
    L22_2 = L22_2 ^ L14_2
    L21_2 = L21_2 - L22_2
    L20_2 = L20_2 / L21_2
    L21_2 = L9_2.bBlack
    L21_2 = L21_2 ^ L14_2
    L22_2 = L9_2.bWhite
    L22_2 = L22_2 ^ L14_2
    L21_2 = L21_2 - L22_2
    L22_2 = L9_2.rBlack
    L22_2 = L22_2 ^ L14_2
    L23_2 = L9_2.rWhite
    L23_2 = L23_2 ^ L14_2
    L22_2 = L22_2 - L23_2
    L21_2 = L21_2 / L22_2
    L22_2 = 1 / L20_2
    L23_2 = L21_2 / L20_2
    L24_2 = 1 / L21_2
    L25_2 = L20_2 / L21_2
    L26_2 = L9_2.rBlack
    L27_2 = L9_2.gBlack
    L28_2 = L9_2.bBlack
    L29_2 = nil
    L30_2 = nil
    L31_2 = nil
    L32_2 = nil
    L33_2 = nil
    L34_2 = nil
    L35_2 = nil
    L36_2 = nil
    L37_2 = nil
    L38_2 = nil
    L39_2 = nil
    L40_2 = nil
    L41_2 = L20_2 - L18_2
    L42_2 = L21_2 - L19_2
    if L41_2 < 0 and L41_2 < L42_2 then
      L43_2 = originalOrigins
      L43_2 = L43_2.greenChannel
      L43_2 = L43_2.whitePointOrigin
      L31_2 = L43_2 ^ L14_2
      L43_2 = L27_2 ^ L14_2
      L43_2 = L43_2 - L31_2
      L44_2 = L26_2 ^ L14_2
      L45_2 = L43_2 * L22_2
      L29_2 = L44_2 - L45_2
      L44_2 = L28_2 ^ L14_2
      L45_2 = L43_2 * L23_2
      L33_2 = L44_2 - L45_2
      L44_2 = originalOrigins
      L44_2 = L44_2.greenChannel
      L44_2 = L44_2.blackPointOrigin
      L32_2 = L44_2 ^ L14_2
      L44_2 = L32_2 - L31_2
      L45_2 = L44_2 * L22_2
      L30_2 = L29_2 + L45_2
      L45_2 = L44_2 * L23_2
      L34_2 = L33_2 + L45_2
    elseif L42_2 < 0 and L41_2 > L42_2 then
      L43_2 = originalOrigins
      L43_2 = L43_2.blueChannel
      L43_2 = L43_2.whitePointOrigin
      L33_2 = L43_2 ^ L14_2
      L43_2 = L28_2 ^ L14_2
      L43_2 = L43_2 - L33_2
      L44_2 = L26_2 ^ L14_2
      L45_2 = L43_2 * L24_2
      L29_2 = L44_2 - L45_2
      L44_2 = L27_2 ^ L14_2
      L45_2 = L43_2 * L25_2
      L31_2 = L44_2 - L45_2
      L44_2 = originalOrigins
      L44_2 = L44_2.blueChannel
      L44_2 = L44_2.blackPointOrigin
      L34_2 = L44_2 ^ L14_2
      L44_2 = L34_2 - L33_2
      L45_2 = L44_2 * L24_2
      L30_2 = L29_2 + L45_2
      L45_2 = L44_2 * L25_2
      L32_2 = L31_2 + L45_2
    else
      L43_2 = originalOrigins
      L43_2 = L43_2.redChannel
      L43_2 = L43_2.whitePointOrigin
      L29_2 = L43_2 ^ L14_2
      L43_2 = L26_2 ^ L14_2
      L43_2 = L43_2 - L29_2
      L44_2 = L27_2 ^ L14_2
      L45_2 = L43_2 * L20_2
      L31_2 = L44_2 - L45_2
      L44_2 = L28_2 ^ L14_2
      L45_2 = L43_2 * L21_2
      L33_2 = L44_2 - L45_2
      L44_2 = originalOrigins
      L44_2 = L44_2.redChannel
      L44_2 = L44_2.blackPointOrigin
      L30_2 = L44_2 ^ L14_2
      L44_2 = L30_2 - L29_2
      L45_2 = L44_2 * L20_2
      L32_2 = L31_2 + L45_2
      L45_2 = L44_2 * L21_2
      L34_2 = L33_2 + L45_2
    end
    if 0 < L29_2 then
      L43_2 = 1 / L14_2
      L35_2 = L29_2 ^ L43_2
    else
      L35_2 = 0
    end
    if 0 < L31_2 then
      L43_2 = 1 / L14_2
      L37_2 = L31_2 ^ L43_2
    else
      L37_2 = 0
    end
    if 0 < L33_2 then
      L43_2 = 1 / L14_2
      L39_2 = L33_2 ^ L43_2
    else
      L39_2 = 0
    end
    L43_2 = 1 / L14_2
    L36_2 = L30_2 ^ L43_2
    L43_2 = 1 / L14_2
    L38_2 = L32_2 ^ L43_2
    L43_2 = 1 / L14_2
    L40_2 = L34_2 ^ L43_2
    L43_2 = originalOrigins
    L43_2 = L43_2.redChannel
    L43_2 = L43_2.blackPointOrigin
    L43_2 = L26_2 - L43_2
    L44_2 = originalOrigins
    L44_2 = L44_2.greenChannel
    L44_2 = L44_2.blackPointOrigin
    L44_2 = L27_2 - L44_2
    L45_2 = originalOrigins
    L45_2 = L45_2.blueChannel
    L45_2 = L45_2.blackPointOrigin
    L45_2 = L28_2 - L45_2
    L46_2 = nil
    L47_2 = nil
    L48_2 = nil
    if L43_2 < 0 then
      L49_2 = originalOrigins
      L49_2 = L49_2.redChannel
      L46_2 = L49_2.blackPointOrigin
    else
      L46_2 = L26_2
    end
    if L44_2 < 0 then
      L49_2 = originalOrigins
      L49_2 = L49_2.greenChannel
      L47_2 = L49_2.blackPointOrigin
    else
      L47_2 = L27_2
    end
    if L45_2 < 0 then
      L49_2 = originalOrigins
      L49_2 = L49_2.blueChannel
      L48_2 = L49_2.blackPointOrigin
    else
      L48_2 = L28_2
    end
    L49_2 = {}
    L49_2.blackPointOrigin = L36_2
    L49_2.whitePointOrigin = L35_2
    L50_2 = L9_2.rMean
    L49_2.meanPoint = L50_2
    L49_2.settingName = "ToneCurvePV2012Red"
    newRedChannel = L49_2
    L49_2 = {}
    L49_2.blackPointOrigin = L38_2
    L49_2.whitePointOrigin = L37_2
    L50_2 = L9_2.gMean
    L49_2.meanPoint = L50_2
    L49_2.settingName = "ToneCurvePV2012Green"
    newGreenChannel = L49_2
    L49_2 = {}
    L49_2.blackPointOrigin = L40_2
    L49_2.whitePointOrigin = L39_2
    L50_2 = L9_2.bMean
    L49_2.meanPoint = L50_2
    L49_2.settingName = "ToneCurvePV2012Blue"
    newBlueChannel = L49_2
    L49_2 = {}
    L50_2 = newRedChannel
    L51_2 = newGreenChannel
    L52_2 = newBlueChannel
    L49_2[1] = L50_2
    L49_2[2] = L51_2
    L49_2[3] = L52_2
    L3_2 = L49_2
  else
    L14_2 = A0_2.rollMode
    if L14_2 == "lab" then
      if L10_2 == nil then
        L14_2 = L33_1
        L15_2 = L9_2
        L16_2 = A1_2
        L14_2 = L14_2(L15_2, L16_2)
        L3_2 = L14_2
      elseif 3 <= L10_2 then
        L14_2 = L34_1
        L15_2 = L9_2
        L16_2 = A1_2
        L14_2 = L14_2(L15_2, L16_2)
        L3_2 = L14_2
      end
    else
      L14_2 = A0_2.rollMode
      if L14_2 == "paper" then
        if L10_2 == nil then
          L14_2 = L33_1
          L15_2 = L9_2
          L16_2 = A1_2
          L14_2 = L14_2(L15_2, L16_2)
          L3_2 = L14_2
        elseif 3 <= L10_2 then
          L14_2 = L34_1
          L15_2 = L9_2
          L16_2 = A1_2
          L14_2 = L14_2(L15_2, L16_2)
          L3_2 = L14_2
        end
        L14_2 = L3_2[1]
        L14_2 = L14_2.blackPointOrigin
        L15_2 = L9_2.rBlack
        L14_2 = L14_2 - L15_2
        L5_2 = L14_2 / 2
        L14_2 = L9_2.rWhite
        L15_2 = L3_2[1]
        L15_2 = L15_2.whitePointOrigin
        L14_2 = L14_2 - L15_2
        L6_2 = L14_2 / 2
      end
    end
  end
  ::lbl_470::
  L7_2 = RollCache
  L7_2 = L7_2[A1_2]
  L8_2 = {}
  L7_2[L2_2] = L8_2
  L7_2 = RollCache
  L7_2 = L7_2[A1_2]
  L7_2 = L7_2[L2_2]
  L7_2.photoOrigins = L3_2
  L7_2 = RollCache
  L7_2 = L7_2[A1_2]
  L7_2 = L7_2[L2_2]
  L7_2.densityCompensation = L4_2
  L7_2 = RollCache
  L7_2 = L7_2[A1_2]
  L7_2 = L7_2[L2_2]
  L7_2.rollWhiteClip = L6_2
  L7_2 = RollCache
  L7_2 = L7_2[A1_2]
  L7_2 = L7_2[L2_2]
  L7_2.rollBlackClip = L5_2
  L7_2 = L3_2
  L8_2 = L4_2
  L9_2 = L6_2
  L10_2 = L5_2
  return L7_2, L8_2, L9_2, L10_2
end

L35_1.getOriginsOfPhotoForRoll = L36_1
L35_1 = NLPRolls

function L36_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L7_1
  L3_2 = L3_2.postAsyncTaskWithContext
  L4_2 = "rollDialog"
  
  function L5_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3
    L1_3 = L6_1
    L1_3 = L1_3.makePropertyTable
    L2_3 = A0_3
    L1_3 = L1_3(L2_3)
    rollProperties = L1_3
    L1_3 = rollProperties
    L2_3 = A0_2
    L2_3 = L2_3.rollAnalysis
    if not L2_3 then
      L2_3 = "add"
    end
    L1_3.rollAction = L2_3
    L1_3 = rollProperties
    L2_3 = _G
    L2_3 = L2_3.activePhotos
    L2_3 = #L2_3
    if 1 < L2_3 then
      L2_3 = "selection"
      if L2_3 then
        goto lbl_23
      end
    end
    L2_3 = "single"
    ::lbl_23::
    L1_3.rollSource = L2_3
    L1_3 = rollProperties
    L1_3.group = 1
    L1_3 = rollProperties
    L1_3.newGroupName = ""
    L1_3 = rollProperties
    L2_3 = {}
    L1_3.groupDropdown = L2_3
    L1_3 = rollProperties
    L1_3.validName = false
    L1_3 = rollProperties
    L1_3.validGroupName = false
    L1_3 = rollProperties
    L1_3.rollName = "Untitled Roll"
    L1_3 = rollProperties
    L2_3 = L29_1
    L2_3 = L2_3()
    L1_3.groupDropdown = L2_3
    L1_3 = L5_1
    L1_3 = L1_3.osFactory
    L1_3 = L1_3()
    L2_3 = rollProperties
    L3_3 = L2_3
    L2_3 = L2_3.addObserver
    L4_3 = "group"
    
    function L5_3(A0_4, A1_4, A2_4)
      local L3_4, L4_4, L5_4, L6_4, L7_4
      if A2_4 ~= "new" then
        return
      end
      L3_4 = L4_1
      L3_4 = L3_4.presentModalDialog
      L4_4 = {}
      L4_4.title = "New Group"
      L5_4 = L31_1
      L6_4 = L1_3
      L7_4 = A0_4
      L5_4 = L5_4(L6_4, L7_4)
      L4_4.contents = L5_4
      L5_4 = {}
      L6_4 = {}
      L6_4.bind_to_object = A0_4
      L6_4.key = "validGroupName"
      L5_4.enabled = L6_4
      L4_4.actionBinding = L5_4
      L4_4.actionVerb = "Create"
      L3_4 = L3_4(L4_4)
      if L3_4 == "ok" then
        L4_4 = A0_4.groupDropdown
        L4_4 = #L4_4
        L4_4 = L4_4 - 1
        L5_4 = A0_4.newGroupName
        L6_4 = L29_1
        L7_4 = {}
        L7_4.title = L5_4
        L7_4.value = L4_4
        L6_4 = L6_4(L7_4)
        A0_4.groupDropdown = L6_4
        A0_4.group = L4_4
      else
        A0_4.group = 1
      end
    end
    
    L2_3(L3_3, L4_3, L5_3)
    L2_3 = "New Roll"
    L3_3 = "Create"
    L4_3 = L30_1
    L5_3 = L1_3
    L6_3 = rollProperties
    L4_3 = L4_3(L5_3, L6_3)
    L5_3 = L4_1
    L5_3 = L5_3.presentModalDialog
    L6_3 = {}
    L6_3.title = L2_3
    L6_3.contents = L4_3
    L7_3 = {}
    L8_3 = {}
    L9_3 = rollProperties
    L8_3.bind_to_object = L9_3
    L8_3.key = "validName"
    L7_3.enabled = L8_3
    L6_3.actionBinding = L7_3
    L6_3.actionVerb = L3_3
    L6_3.save_frame = "NLPRollsFrame"
    L5_3 = L5_3(L6_3)
    if L5_3 == "ok" then
      L6_3 = nil
      L7_3 = tonumber
      L8_3 = rollProperties
      L8_3 = L8_3.group
      L7_3 = L7_3(L8_3)
      L8_3 = rollProperties
      L8_3 = L8_3.groupDropdown
      L8_3 = L8_3[L7_3]
      L6_3 = L8_3.title
      L8_3 = _G
      L8_3 = L8_3.rolls
      L8_3 = L8_3.groups
      L8_3[L6_3] = true
      L8_3 = rollProperties
      L8_3 = L8_3.rollSource
      if L8_3 == "selection" then
        L8_3 = NLPRolls
        L8_3 = L8_3.createRoll
        L9_3 = A1_2
        L10_3 = rollProperties
        L10_3 = L10_3.rollName
        L11_3 = L6_3
        L12_3 = true
        L8_3(L9_3, L10_3, L11_3, L12_3)
      else
        L8_3 = rollProperties
        L8_3 = L8_3.rollSource
        if L8_3 == "single" then
          L8_3 = NLPRolls
          L8_3 = L8_3.createRoll
          L9_3 = {}
          L10_3 = _G
          L10_3 = L10_3.activePhotos
          L11_3 = _G
          L11_3 = L11_3.activePhotoIndex
          L10_3 = L10_3[L11_3]
          L9_3[1] = L10_3
          L10_3 = rollProperties
          L10_3 = L10_3.rollName
          L11_3 = L6_3
          L12_3 = true
          L8_3(L9_3, L10_3, L11_3, L12_3)
        else
          L8_3 = rollProperties
          L8_3 = L8_3.rollSource
          if L8_3 == "folder" then
          end
        end
      end
    else
      L6_3 = A2_2
      if L6_3 ~= nil then
        L6_3 = A0_2
        L7_3 = A2_2
        L6_3.rollAnalysis = L7_3
      end
    end
  end
  
  L3_2(L4_2, L5_2)
end

L35_1.loadRollDialog = L36_1
L35_1 = NLPRolls

function L36_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L7_1
  L3_2 = L3_2.postAsyncTaskWithContext
  L4_2 = "manageRollDialog"
  
  function L5_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3
    L1_3 = L5_1
    L1_3 = L1_3.osFactory
    L1_3 = L1_3()
    L2_3 = L6_1
    L2_3 = L2_3.makePropertyTable
    L3_3 = A0_3
    L2_3 = L2_3(L3_3)
    L3_3 = {}
    L4_3 = next
    L5_3 = _G
    L5_3 = L5_3.rolls
    L4_3 = L4_3(L5_3)
    if L4_3 ~= nil then
      L4_3 = next
      L5_3 = _G
      L5_3 = L5_3.rolls
      L5_3 = L5_3.groups
      L4_3 = L4_3(L5_3)
      if L4_3 ~= nil then
        L4_3 = {}
        L5_3 = pairs
        L6_3 = _G
        L6_3 = L6_3.rolls
        L6_3 = L6_3.groups
        L5_3, L6_3, L7_3 = L5_3(L6_3)
        for L8_3 in L5_3, L6_3, L7_3 do
          L9_3 = table
          L9_3 = L9_3.insert
          L10_3 = L4_3
          L11_3 = L8_3
          L9_3(L10_3, L11_3)
        end
        L5_3 = table
        L5_3 = L5_3.sort
        L6_3 = L4_3
        L5_3(L6_3)
        L5_3 = ipairs
        L6_3 = L4_3
        L5_3, L6_3, L7_3 = L5_3(L6_3)
        for L8_3, L9_3 in L5_3, L6_3, L7_3 do
          L10_3 = L9_3
          L11_3 = _G
          L11_3 = L11_3.rolls
          L11_3 = L11_3.groups
          L11_3 = L11_3[L10_3]
          L2_3[L10_3] = L11_3
          L12_3 = L1_3
          L11_3 = L1_3.checkbox
          L13_3 = {}
          L13_3.title = L10_3
          L13_3.size = "small"
          L14_3 = L5_1
          L14_3 = L14_3.bind
          L15_3 = L10_3
          L14_3 = L14_3(L15_3)
          L13_3.value = L14_3
          L11_3 = L11_3(L12_3, L13_3)
          L12_3 = table
          L12_3 = L12_3.insert
          L13_3 = L3_3
          L14_3 = L11_3
          L12_3(L13_3, L14_3)
        end
    end
    else
      L4_3 = table
      L4_3 = L4_3.insert
      L5_3 = L3_3
      L7_3 = L1_3
      L6_3 = L1_3.static_text
      L8_3 = {}
      L8_3.title = "Nothing to manage yet"
      L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3 = L6_3(L7_3, L8_3)
      L4_3(L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3)
    end
    L5_3 = L1_3
    L4_3 = L1_3.scrolled_view
    L6_3 = {}
    L8_3 = L1_3
    L7_3 = L1_3.control_spacing
    L7_3 = L7_3(L8_3)
    L6_3.spacing = L7_3
    L6_3.horizontal_scroller = false
    L6_3.vertical_scroller = true
    L6_3.height = 400
    L6_3.width = 400
    L6_3.margin = 5
    L7_3 = L11_1
    L8_3 = 0.85
    L9_3 = 0.85
    L10_3 = 0.85
    L7_3 = L7_3(L8_3, L9_3, L10_3)
    L6_3.background_color = L7_3
    L6_3.bind_to_object = L2_3
    L8_3 = L1_3
    L7_3 = L1_3.column
    L9_3 = L3_3
    L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3 = L7_3(L8_3, L9_3)
    L6_3[1] = L7_3
    L6_3[2] = L8_3
    L6_3[3] = L9_3
    L6_3[4] = L10_3
    L6_3[5] = L11_3
    L6_3[6] = L12_3
    L6_3[7] = L13_3
    L6_3[8] = L14_3
    L6_3[9] = L15_3
    L4_3 = L4_3(L5_3, L6_3)
    L6_3 = L1_3
    L5_3 = L1_3.row
    L7_3 = {}
    L9_3 = L1_3
    L8_3 = L1_3.push_button
    L10_3 = {}
    L10_3.title = "Check All"
    
    function L11_3()
      local L0_4, L1_4, L2_4, L3_4, L4_4, L5_4
      L0_4 = L2_3
      L1_4 = L0_4
      L0_4 = L0_4.pairs
      L0_4, L1_4, L2_4 = L0_4(L1_4)
      for L3_4, L4_4 in L0_4, L1_4, L2_4 do
        L5_4 = L2_3
        L5_4[L3_4] = true
      end
    end
    
    L10_3.action = L11_3
    L8_3 = L8_3(L9_3, L10_3)
    L10_3 = L1_3
    L9_3 = L1_3.push_button
    L11_3 = {}
    L11_3.title = "Check None"
    
    function L12_3()
      local L0_4, L1_4, L2_4, L3_4, L4_4, L5_4
      L0_4 = L2_3
      L1_4 = L0_4
      L0_4 = L0_4.pairs
      L0_4, L1_4, L2_4 = L0_4(L1_4)
      for L3_4, L4_4 in L0_4, L1_4, L2_4 do
        L5_4 = L2_3
        L5_4[L3_4] = false
      end
    end
    
    L11_3.action = L12_3
    L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3 = L9_3(L10_3, L11_3)
    L7_3[1] = L8_3
    L7_3[2] = L9_3
    L7_3[3] = L10_3
    L7_3[4] = L11_3
    L7_3[5] = L12_3
    L7_3[6] = L13_3
    L7_3[7] = L14_3
    L7_3[8] = L15_3
    L5_3 = L5_3(L6_3, L7_3)
    L6_3 = L4_1
    L6_3 = L6_3.presentModalDialog
    L7_3 = {}
    L7_3.title = "Manage Roll Groups"
    L7_3.contents = L4_3
    L7_3.accessoryView = L5_3
    L7_3.actionVerb = "Save"
    L6_3 = L6_3(L7_3)
    if L6_3 == "ok" then
      L7_3 = _G
      L7_3.rollAdded = true
      L7_3 = _G
      L7_3 = L7_3.rolls
      L8_3 = L1_1
      L8_3 = L8_3.copyObservableTable
      L9_3 = L2_3
      L8_3 = L8_3(L9_3)
      L7_3.groups = L8_3
      L7_3 = createRollDropdown
      L7_3()
      L7_3 = A2_2
      if L7_3 ~= nil then
        L7_3 = A0_2
        L8_3 = A2_2
        L7_3.rollAnalysis = L8_3
      end
    else
      L7_3 = A2_2
      if L7_3 ~= nil then
        L7_3 = A0_2
        L8_3 = A2_2
        L7_3.rollAnalysis = L8_3
      end
    end
  end
  
  L3_2(L4_2, L5_2)
end

L35_1.loadManageRollsDialog = L36_1
L35_1 = NLPRolls

function L36_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  if A1_2 ~= nil then
  end
  L2_2 = false
  L3_2 = nil
  L4_2 = "rollID_"
  L5_2 = A1_2
  L4_2 = L4_2 .. L5_2
  L5_2 = _G
  L5_2 = L5_2.activePhotosMeta
  L6_2 = activePhotoIndex
  L5_2 = L5_2[L6_2]
  L5_2 = L5_2.roll
  if L5_2 ~= nil then
    L5_2 = _G
    L5_2 = L5_2.activePhotosMeta
    L6_2 = activePhotoIndex
    L5_2 = L5_2[L6_2]
    L5_2 = L5_2.roll
    L5_2 = L5_2.ID
    if L5_2 == A1_2 then
      L2_2 = true
      L5_2 = _G
      L5_2 = L5_2.activePhotosMeta
      L6_2 = activePhotoIndex
      L5_2 = L5_2[L6_2]
      L5_2 = L5_2.roll
      L3_2 = L5_2.rollName
  end
  else
    L5_2 = _G
    L5_2 = L5_2.rolls
    L5_2 = L5_2[L4_2]
    if L5_2 ~= nil then
      L5_2 = _G
      L5_2 = L5_2.rolls
      L5_2 = L5_2[L4_2]
      L3_2 = L5_2.rollName
    else
    end
  end
  L5_2 = "Delete \""
  L6_2 = L3_2
  L7_2 = "?\""
  L5_2 = L5_2 .. L6_2 .. L7_2
  L6_2 = "Are you sure you want to delete the roll \""
  L7_2 = L3_2
  L8_2 = "?\" This could make the roll unavailabe to other negatives which are currently using it."
  L6_2 = L6_2 .. L7_2 .. L8_2
  L7_2 = "Yes, Delete This Roll"
  L8_2 = "Cancel"
  L9_2 = L4_1
  L9_2 = L9_2.confirm
  L10_2 = L5_2
  L11_2 = L6_2
  L12_2 = L7_2
  L13_2 = L8_2
  L9_2 = L9_2(L10_2, L11_2, L12_2, L13_2)
  if L9_2 == "ok" then
    if L2_2 == true then
      L10_2 = _G
      L10_2 = L10_2.activePhotosMeta
      L11_2 = activePhotoIndex
      L10_2 = L10_2[L11_2]
      L10_2.roll = nil
      L10_2 = true
      deleteAttachedRoll = L10_2
    else
      L10_2 = _G
      L10_2 = L10_2.rolls
      L10_2[L4_2] = nil
      L10_2 = _G
      L10_2.rollRemoved = true
    end
    L10_2 = createRollDropdown
    L10_2()
    A0_2.rollAnalysis = "individual"
  else
    A0_2.rollAnalysis = A1_2
  end
end

L35_1.deleteRoll = L36_1
L35_1 = NLPRolls

function L36_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  if A1_2 ~= nil then
  end
  L2_2 = false
  L3_2 = nil
  L4_2 = nil
  L5_2 = "rollID_"
  L6_2 = A1_2
  L5_2 = L5_2 .. L6_2
  L6_2 = _G
  L6_2 = L6_2.activePhotosMeta
  L7_2 = activePhotoIndex
  L6_2 = L6_2[L7_2]
  L6_2 = L6_2.roll
  if L6_2 ~= nil then
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L7_2 = activePhotoIndex
    L6_2 = L6_2[L7_2]
    L6_2 = L6_2.roll
    L6_2 = L6_2.ID
    if L6_2 == A1_2 then
      L2_2 = true
      L6_2 = _G
      L6_2 = L6_2.activePhotosMeta
      L7_2 = activePhotoIndex
      L6_2 = L6_2[L7_2]
      L6_2 = L6_2.roll
      L3_2 = L6_2.rollName
      L4_2 = "none"
    end
  end
  L6_2 = _G
  L6_2 = L6_2.rolls
  L6_2 = L6_2[L5_2]
  if L6_2 ~= nil then
    L6_2 = _G
    L6_2 = L6_2.rolls
    L6_2 = L6_2[L5_2]
    L3_2 = L6_2.rollName
    L6_2 = _G
    L6_2 = L6_2.rolls
    L6_2 = L6_2[L5_2]
    L4_2 = L6_2.groupName
    L6_2 = _G
    L6_2.rollAdded = true
  end
  L6_2 = L7_1
  L6_2 = L6_2.postAsyncTaskWithContext
  L7_2 = "editRollDialog"
  
  function L8_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3
    L1_3 = L6_1
    L1_3 = L1_3.makePropertyTable
    L2_3 = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = L3_2
    L1_3.rollName = L2_3
    L1_3.validRollName = true
    L2_3 = L4_2
    L1_3.groupName = L2_3
    L2_3 = L29_1
    L2_3 = L2_3()
    L1_3.groupDropdown = L2_3
    L2_3 = L5_1
    L2_3 = L2_3.osFactory
    L2_3 = L2_3()
    L4_3 = L2_3
    L3_3 = L2_3.column
    L5_3 = {}
    L5_3.bind_to_object = L1_3
    L7_3 = L2_3
    L6_3 = L2_3.row
    L8_3 = {}
    L10_3 = L2_3
    L9_3 = L2_3.static_text
    L11_3 = {}
    L11_3.title = "Roll Name:"
    L11_3.alignment = "right"
    L12_3 = L5_1
    L12_3 = L12_3.share
    L13_3 = "edit_roll_label_width"
    L12_3 = L12_3(L13_3)
    L11_3.width = L12_3
    L9_3 = L9_3(L10_3, L11_3)
    L11_3 = L2_3
    L10_3 = L2_3.edit_field
    L12_3 = {}
    L12_3.immediate = true
    L12_3.alignment = "left"
    L12_3.width = 150
    L13_3 = L5_1
    L13_3 = L13_3.bind
    L14_3 = "rollName"
    L13_3 = L13_3(L14_3)
    L12_3.value = L13_3
    
    function L13_3(A0_4, A1_4)
      local L2_4, L3_4
      L2_4 = string
      L2_4 = L2_4.len
      L3_4 = A1_4
      L2_4 = L2_4(L3_4)
      if 0 < L2_4 then
        L2_4 = L1_3
        L2_4.validRollName = true
        L2_4 = true
        L3_4 = A1_4
        return L2_4, L3_4
      else
        L2_4 = L1_3
        L2_4.validRollName = false
        L2_4 = false
        L3_4 = A1_4
        return L2_4, L3_4
      end
    end
    
    L12_3.validate = L13_3
    L10_3, L11_3, L12_3, L13_3, L14_3 = L10_3(L11_3, L12_3)
    L8_3[1] = L9_3
    L8_3[2] = L10_3
    L8_3[3] = L11_3
    L8_3[4] = L12_3
    L8_3[5] = L13_3
    L8_3[6] = L14_3
    L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3 = L6_3(L7_3, L8_3)
    L5_3[1] = L6_3
    L5_3[2] = L7_3
    L5_3[3] = L8_3
    L5_3[4] = L9_3
    L5_3[5] = L10_3
    L5_3[6] = L11_3
    L5_3[7] = L12_3
    L5_3[8] = L13_3
    L5_3[9] = L14_3
    L3_3 = L3_3(L4_3, L5_3)
    L4_3 = L4_1
    L4_3 = L4_3.presentModalDialog
    L5_3 = {}
    L5_3.title = "Rename Roll"
    L5_3.contents = L3_3
    L6_3 = {}
    L7_3 = {}
    L7_3.bind_to_object = L1_3
    L7_3.key = "validRollName"
    L6_3.enabled = L7_3
    L5_3.actionBinding = L6_3
    L5_3.actionVerb = "Update"
    L4_3 = L4_3(L5_3)
    if L4_3 == "ok" then
      L5_3 = L2_2
      if L5_3 == true then
        L5_3 = _G
        L5_3 = L5_3.activePhotosMeta
        L6_3 = activePhotoIndex
        L5_3 = L5_3[L6_3]
        L5_3 = L5_3.roll
        L6_3 = L1_3.rollName
        L5_3.rollName = L6_3
        L5_3 = _G
        L5_3 = L5_3.activePhotosMeta
        L6_3 = activePhotoIndex
        L5_3 = L5_3[L6_3]
        L6_3 = L1_3.rollName
        L5_3.rollName = L6_3
        L5_3 = A0_2
        L6_3 = _G
        L6_3 = L6_3.activePhotosMeta
        L7_3 = activePhotoIndex
        L6_3 = L6_3[L7_3]
        L6_3 = L6_3.roll
        L6_3 = L6_3.ID
        L5_3.rollAnalysis = L6_3
      end
      L5_3 = _G
      L5_3 = L5_3.rolls
      L6_3 = L5_2
      L5_3 = L5_3[L6_3]
      if L5_3 ~= nil then
        L5_3 = _G
        L5_3 = L5_3.rolls
        L6_3 = L5_2
        L5_3 = L5_3[L6_3]
        L6_3 = L1_3.rollName
        L5_3.rollName = L6_3
        L5_3 = _G
        L5_3.rollAdded = true
      end
      L5_3 = A0_2
      L6_3 = A1_2
      L5_3.rollAnalysis = L6_3
      L5_3 = createRollDropdown
      L5_3()
    else
      L5_3 = A0_2
      L6_3 = A1_2
      L5_3.rollAnalysis = L6_3
    end
  end
  
  L6_2(L7_2, L8_2)
end

L35_1.editRoll = L36_1
L35_1 = NLPRolls

function L36_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L5_2 = {}
  L6_2 = #A0_2
  L5_2.currentSize = L6_2
  L5_2.rollName = A1_2
  L6_2 = A2_2 or L6_2
  if not A2_2 then
    L6_2 = "User"
  end
  L5_2.groupName = L6_2
  L6_2 = L3_1
  L6_2 = L6_2.generateUUID
  L6_2 = L6_2()
  L5_2.ID = L6_2
  L5_2.processVersion = 3
  L6_2 = nil
  L7_2 = nil
  L8_2 = nil
  L9_2 = nil
  L10_2 = nil
  L11_2 = nil
  L12_2 = nil
  L13_2 = nil
  L14_2 = L21_1
  L15_2 = A0_2
  L16_2 = A3_2
  L14_2 = L14_2(L15_2, L16_2)
  L6_2 = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.combineTablesByKey
  L15_2 = L6_2
  L14_2 = L14_2(L15_2)
  L7_2 = L14_2
  L14_2 = L15_1
  if L14_2 == true then
    L14_2 = L1_1
    L14_2 = L14_2.mode
    L15_2 = L7_2.ev
    L14_2 = L14_2(L15_2)
    L15_2 = L14_2[1]
    L5_2.commonEV = L15_2
    L15_2 = nil
    L16_2 = L22_1
    L17_2 = L7_2
    L18_2 = L5_2.commonEV
    L16_2, L17_2 = L16_2(L17_2, L18_2)
    L15_2 = L17_2
    L7_2 = L16_2
    if L15_2 ~= nil then
      if L15_2 == "cancel" then
        return
      elseif L15_2 == "exposure-off" then
        L5_2.commonEV = 0
      end
    end
  end
  L14_2 = L16_1
  if L14_2 == true then
    L14_2 = L1_1
    L14_2 = L14_2.mode
    L15_2 = L7_2.lrTemp
    L14_2 = L14_2(L15_2)
    L14_2 = L14_2[1]
    L5_2.lrTemp = L14_2
    L14_2 = L1_1
    L14_2 = L14_2.mode
    L15_2 = L7_2.lrTint
    L14_2 = L14_2(L15_2)
    L14_2 = L14_2[1]
    L5_2.lrTint = L14_2
  end
  L14_2 = L1_1
  L14_2 = L14_2.combineTablesByKey
  L15_2 = L7_2.redChannel
  L14_2 = L14_2(L15_2)
  L9_2 = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.combineTablesByKey
  L15_2 = L7_2.greenChannel
  L14_2 = L14_2(L15_2)
  L10_2 = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.combineTablesByKey
  L15_2 = L7_2.blueChannel
  L14_2 = L14_2(L15_2)
  L11_2 = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.min
  L15_2 = L9_2.whitePointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.rWhite = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.min
  L15_2 = L10_2.whitePointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.gWhite = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.min
  L15_2 = L11_2.whitePointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.bWhite = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.max
  L15_2 = L9_2.blackPointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.rBlack = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.max
  L15_2 = L10_2.blackPointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.gBlack = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.max
  L15_2 = L11_2.blackPointOrigin
  L14_2 = L14_2(L15_2)
  L5_2.bBlack = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.rgbAvg
  L15_2 = L9_2.meanPoint
  L14_2 = L14_2(L15_2)
  L5_2.rMean = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.rgbAvg
  L15_2 = L10_2.meanPoint
  L14_2 = L14_2(L15_2)
  L5_2.gMean = L14_2
  L14_2 = L1_1
  L14_2 = L14_2.rgbAvg
  L15_2 = L11_2.meanPoint
  L14_2 = L14_2(L15_2)
  L5_2.bMean = L14_2
  L14_2 = L23_1
  L15_2 = L9_2
  L16_2 = L10_2
  L17_2 = L11_2
  L14_2, L15_2 = L14_2(L15_2, L16_2, L17_2)
  L13_2 = L15_2
  L12_2 = L14_2
  L5_2.blueMultiplier = L12_2
  L5_2.greenMultiplier = L13_2
  L14_2 = L24_1
  L15_2 = L7_2.autoWB
  L14_2, L15_2, L16_2 = L14_2(L15_2)
  L17_2 = {}
  L17_2.smartColorNeutral = L14_2
  L17_2.smartColorWarming = L15_2
  L17_2.smartColorCooling = L16_2
  L5_2.autoWB = L17_2
  L17_2 = L25_1
  L18_2 = L5_2
  L19_2 = A0_2
  L17_2(L18_2, L19_2)
  if A4_2 ~= true then
    L17_2 = L27_1
    L18_2 = L5_2
    L17_2(L18_2)
    L17_2 = L28_1
    L18_2 = L5_2
    L17_2(L18_2)
  end
  L17_2 = L5_2.ID
  return L17_2
end

L35_1.createRoll = L36_1
L35_1 = NLPRolls
return L35_1
