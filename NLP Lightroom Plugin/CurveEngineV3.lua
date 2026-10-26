local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1

L1_1 = {}
L2_1 = require
L3_1 = "inspect"
L2_1 = L2_1(L3_1)
L3_1 = require
L4_1 = "UtilV3"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrLogger"
L4_1 = L4_1(L5_1)
L5_1 = require
L6_1 = "NegativePresets"
L5_1 = L5_1(L6_1)
L6_1 = require
L7_1 = "NLPColorV3"
L6_1 = L6_1(L7_1)
L7_1 = L4_1
L8_1 = "Negative"
L7_1 = L7_1(L8_1)
L8_1 = _G
L8_1 = L8_1.dev
if L8_1 == true then
  L9_1 = L7_1
  L8_1 = L7_1.enable
  L10_1 = "logfile"
  L8_1(L9_1, L10_1)
else
  L9_1 = L7_1
  L8_1 = L7_1.disable
  L8_1(L9_1)
end
L8_1 = 0.1
L9_1 = 0.8

function L10_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L3_2 = {}
  L4_2 = A1_2.toneProfile
  L5_2 = L5_1
  L5_2 = L5_2.toneProfiles
  L5_2 = L5_2[L4_2]
  L6_2 = 1
  L7_2 = L5_2.autoTone
  if L7_2 then
    L7_2 = L1_1
    L7_2 = L7_2.autoToneGamma
    L8_2 = A0_2
    L9_2 = A1_2
    L7_2 = L7_2(L8_2, L9_2)
    L6_2 = L7_2
  end
  L7_2 = 0.01
  L8_2 = A1_2.cyan
  L8_2 = L8_2 * L7_2
  L8_2 = 1 - L8_2
  L3_2.cyan = L8_2
  L8_2 = A1_2.tint
  L8_2 = L8_2 * L7_2
  L8_2 = 1 - L8_2
  L3_2.tint = L8_2
  L8_2 = A1_2.temp
  L8_2 = L8_2 * L7_2
  L8_2 = 1 - L8_2
  L3_2.temp = L8_2
  L8_2 = A1_2.wbTint
  L8_2 = -L8_2
  L9_2 = _G
  L9_2 = L9_2.tempMultiplier
  L9_2 = 255 / L9_2
  L8_2 = L8_2 / L9_2
  L3_2.wbTint = L8_2
  L8_2 = A1_2.wbTemp
  L8_2 = -L8_2
  L9_2 = _G
  L9_2 = L9_2.tempMultiplier
  L9_2 = 255 / L9_2
  L8_2 = L8_2 / L9_2
  L3_2.wbTemp = L8_2
  L8_2 = A1_2.brightness
  L8_2 = L8_2 * 0.02
  L8_2 = 1 + L8_2
  L8_2 = 1 / L8_2
  L9_2 = L5_2.defaultGamma
  L9_2 = 1 / L9_2
  L8_2 = L8_2 * L9_2
  L9_2 = 1 / L6_2
  L8_2 = L8_2 * L9_2
  L3_2.brightness = L8_2
  if A2_2 ~= nil and A2_2 ~= 0 then
    L8_2 = L3_2.brightness
    L9_2 = A2_2 * 0.02
    L9_2 = 1 + L9_2
    L9_2 = 1 / L9_2
    L8_2 = L8_2 * L9_2
    L3_2.brightness = L8_2
  else
  end
  L8_2 = A1_2.exposure
  L8_2 = L8_2 * 0.02
  L8_2 = 1 + L8_2
  L8_2 = 1 / L8_2
  L3_2.exposure = L8_2
  L8_2 = A1_2.contrast
  L9_2 = L5_2.defaultContrast
  L8_2 = L8_2 + L9_2
  L3_2.contrast = L8_2
  L8_2 = A1_2.highlights
  L9_2 = L5_2.defaultHighlights
  L8_2 = L8_2 + L9_2
  L9_2 = A1_2.glow
  L9_2 = L9_2 / 1.5
  L8_2 = L8_2 + L9_2
  L3_2.highlights = L8_2
  L8_2 = A1_2.shadows
  L9_2 = L5_2.defaultShadows
  L8_2 = L8_2 + L9_2
  L9_2 = A1_2.fade
  L9_2 = L9_2 / 1.5
  L8_2 = L8_2 - L9_2
  L3_2.shadows = L8_2
  L8_2 = A1_2.blacks
  L9_2 = L5_2.defaultBlacks
  L8_2 = L8_2 + L9_2
  L9_2 = A1_2.fade
  L8_2 = L8_2 + L9_2
  L3_2.blacks = L8_2
  L8_2 = A1_2.shadowRange
  L3_2.shadowRange = L8_2
  L8_2 = A1_2.shadowCyan
  L3_2.shadowCyan = L8_2
  L8_2 = A1_2.shadowTint
  L3_2.shadowTint = L8_2
  L8_2 = A1_2.shadowTemp
  L3_2.shadowTemp = L8_2
  L8_2 = A1_2.whites
  L9_2 = L5_2.defaultWhites
  L8_2 = L8_2 + L9_2
  L9_2 = A1_2.glow
  L8_2 = L8_2 - L9_2
  L3_2.whites = L8_2
  return L3_2
end

function L11_1(A0_2, A1_2)
  local L2_2, L3_2
  L3_2 = A1_2.curveResolutionType
  if L3_2 == "auto_curve_pts" then
    L3_2 = A1_2.imageType
    if L3_2 ~= "positive" then
      L3_2 = A1_2.imageType
      if L3_2 ~= "editOnly" then
        goto lbl_12
      end
    end
    L2_2 = 7
    goto lbl_40
    ::lbl_12::
    if A0_2 <= 30 then
      L2_2 = 3
    elseif A0_2 <= 70 then
      L2_2 = 5
    elseif A0_2 < 128 then
      L2_2 = 7
    else
      L2_2 = 9
    end
  else
    L3_2 = A1_2.curveResolutionType
    if L3_2 == "smooth_curve_pts" then
      L2_2 = 4
    else
      L3_2 = A1_2.curveResolutionType
      if L3_2 == "precise_curve_pts" then
        L2_2 = 14
      else
        L3_2 = A1_2.curveResolutionType
        if L3_2 == "manual_curve_pts" then
          L2_2 = A1_2.curveResolution
        end
      end
    end
  end
  ::lbl_40::
  return L2_2
end

function L12_1(A0_2)
  local L1_2
  if 1 < A0_2 then
    L1_2 = 1
    return L1_2
  elseif A0_2 < 0 then
    L1_2 = 0
    return L1_2
  else
    return A0_2
  end
end

function L13_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = math
  L3_2 = L3_2.tanh
  L4_2 = 0.5 * A0_2
  L5_2 = A2_2 - A1_2
  L4_2 = L4_2 * L5_2
  return L3_2(L4_2)
end

function L14_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L4_2 = A3_2 or nil
  if not A3_2 then
    L4_2 = 1
  end
  L5_2 = A2_2 / L4_2
  L6_2 = L13_1
  L7_2 = A0_2
  L8_2 = A1_2
  L9_2 = L5_2
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  L7_2 = L13_1
  L8_2 = A0_2
  L9_2 = A1_2
  L10_2 = 0
  L7_2 = L7_2(L8_2, L9_2, L10_2)
  L6_2 = L6_2 - L7_2
  L7_2 = L13_1
  L8_2 = A0_2
  L9_2 = A1_2
  L10_2 = 1
  L7_2 = L7_2(L8_2, L9_2, L10_2)
  L8_2 = L13_1
  L9_2 = A0_2
  L10_2 = A1_2
  L11_2 = 0
  L8_2 = L8_2(L9_2, L10_2, L11_2)
  L7_2 = L7_2 - L8_2
  L6_2 = L6_2 / L7_2
  L6_2 = L6_2 * L4_2
  return L6_2
end

function L15_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L4_2 = A3_2 or nil
  if not A3_2 then
    L4_2 = 1
  end
  L5_2 = L13_1
  L6_2 = A0_2
  L7_2 = A1_2
  L8_2 = 0
  L5_2 = L5_2(L6_2, L7_2, L8_2)
  L6_2 = L13_1
  L7_2 = A0_2
  L8_2 = A1_2
  L9_2 = 1
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  L7_2 = L6_2 - L5_2
  L8_2 = A2_2 / L4_2
  L7_2 = L7_2 * L8_2
  L7_2 = L7_2 + L5_2
  L8_2 = L7_1
  L9_2 = L8_2
  L8_2 = L8_2.trace
  L10_2 = "Inverted sigmoidal argument: "
  L11_2 = L7_2
  L10_2 = L10_2 .. L11_2
  L8_2(L9_2, L10_2)
  if 1 < L7_2 then
    L7_2 = 1
  elseif L7_2 < -1 then
    L7_2 = -1
  end
  L8_2 = 2 / A0_2
  L9_2 = L3_1
  L9_2 = L9_2.atanh
  L10_2 = L7_2
  L9_2 = L9_2(L10_2)
  L8_2 = L8_2 * L9_2
  L8_2 = A1_2 + L8_2
  L8_2 = L8_2 * L4_2
  L9_2 = L7_1
  L10_2 = L9_2
  L9_2 = L9_2.trace
  L11_2 = "Inverted sigmoidal result: "
  L12_2 = L8_2
  L11_2 = L11_2 .. L12_2
  L9_2(L10_2, L11_2)
  return L8_2
end

function L16_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L3_2 = A0_2[1]
  L3_2 = L3_2.meanPoint
  L4_2 = A0_2[2]
  L4_2 = L4_2.meanPoint
  L5_2 = A0_2[3]
  L5_2 = L5_2.meanPoint
  L6_2 = L3_2 + L4_2
  L6_2 = L6_2 + L5_2
  L6_2 = L6_2 / 3
  L7_2 = 0.5
  L8_2 = A1_2.imageType
  if L8_2 ~= "positive" then
    L8_2 = A1_2.imageType
    if L8_2 ~= "editOnly" then
      goto lbl_24
    end
  end
  L8_2 = L3_1
  L8_2 = L8_2.logb
  L9_2 = L7_2
  L10_2 = L6_2
  L8_2 = L8_2(L9_2, L10_2)
  L2_2 = L8_2
  goto lbl_30
  ::lbl_24::
  L8_2 = L3_1
  L8_2 = L8_2.logb
  L9_2 = L7_2
  L10_2 = L6_2
  L8_2 = L8_2(L9_2, L10_2)
  L2_2 = 1 / L8_2
  ::lbl_30::
  if 1.1 < L2_2 then
    L2_2 = 1.1
  elseif L2_2 < 0.8 then
    L2_2 = 0.8
  end
  return L2_2
end

L1_1.autoToneGamma = L16_1

function L16_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2
  L3_2 = A0_2
  L4_2 = nil
  L5_2 = nil
  L6_2 = nil
  L7_2 = nil
  L8_2 = nil
  L9_2 = nil
  L10_2 = 1
  L11_2 = 3
  L12_2 = 1
  for L13_2 = L10_2, L11_2, L12_2 do
    L6_2 = A0_2[L13_2]
    L14_2 = A0_2[L13_2]
    L15_2 = A2_2 - 1
    L4_2 = L14_2[L15_2]
    L14_2 = A0_2[L13_2]
    L5_2 = L14_2[A2_2]
    L7_2 = L5_2 - L4_2
    L14_2 = A1_2 + 1
    L14_2 = L7_2 / L14_2
    incrementSize = L14_2
    L14_2 = 1
    L15_2 = A1_2
    L16_2 = 1
    for L17_2 = L14_2, L15_2, L16_2 do
      L18_2 = incrementSize
      L18_2 = L18_2 * L17_2
      L9_2 = L4_2 + L18_2
      L18_2 = table
      L18_2 = L18_2.insert
      L19_2 = L3_2[L13_2]
      L20_2 = A2_2 + L17_2
      L20_2 = L20_2 - 1
      L21_2 = L9_2
      L18_2(L19_2, L20_2, L21_2)
    end
  end
  return L3_2
end

function L17_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L1_2 = {}
  L2_2 = L16_1
  L3_2 = {}
  L4_2 = {}
  L5_2 = 0
  L6_2 = 1
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L5_2 = {}
  L6_2 = 0
  L7_2 = 1
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L6_2 = {}
  L7_2 = 0
  L8_2 = 1
  L6_2[1] = L7_2
  L6_2[2] = L8_2
  L3_2[1] = L4_2
  L3_2[2] = L5_2
  L3_2[3] = L6_2
  L4_2 = A0_2 - 2
  L5_2 = 2
  L2_2 = L2_2(L3_2, L4_2, L5_2)
  L1_2 = L2_2
  return L1_2
end

function L18_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2
  L2_2 = {}
  L3_2 = 1
  L4_2 = #A0_2
  L5_2 = 1
  for L6_2 = L3_2, L4_2, L5_2 do
    L7_2 = {}
    L2_2[L6_2] = L7_2
    L7_2 = 1
    L8_2 = A0_2[L6_2]
    L8_2 = #L8_2
    L9_2 = 1
    for L10_2 = L7_2, L8_2, L9_2 do
      L11_2 = L2_2[L6_2]
      L12_2 = A1_2
      L13_2 = A0_2[L6_2]
      L13_2 = L13_2[L10_2]
      L14_2 = L6_2
      L15_2 = L10_2
      L12_2 = L12_2(L13_2, L14_2, L15_2)
      L11_2[L10_2] = L12_2
    end
  end
  return L2_2
end

function L19_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L18_1
  L4_2 = A0_2
  
  function L5_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3
    L3_3 = L12_1
    L4_3 = A1_2
    L4_3 = L4_3[A1_3]
    L4_3 = L4_3[A2_3]
    L4_3 = A0_3 + L4_3
    L5_3 = A2_2
    L5_3 = L5_3[A1_3]
    L5_3 = L5_3[A2_3]
    L4_3 = L4_3 - L5_3
    return L3_3(L4_3)
  end
  
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = L9_1
  L2_2 = 1 - L2_2
  L3_2 = L18_1
  L4_2 = A0_2
  
  function L5_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3
    L3_3 = A1_2
    L3_3 = L3_3[A1_3]
    L3_3 = A0_3 - L3_3
    L4_3 = L9_1
    if L3_3 > L4_3 then
      L4_3 = L2_2
      L4_3 = L4_3 * A0_3
      L5_3 = L2_2
      L6_3 = L9_1
      L5_3 = L5_3 * L6_3
      L4_3 = L4_3 - L5_3
      L5_3 = L2_2
      L6_3 = A1_2
      L6_3 = L6_3[A1_3]
      L5_3 = L5_3 * L6_3
      L4_3 = L4_3 - L5_3
      L5_3 = L2_2
      L6_3 = A1_2
      L6_3 = L6_3[A1_3]
      L5_3 = L5_3 - L6_3
      L4_3 = L4_3 / L5_3
      L5_3 = L9_1
      L4_3 = L4_3 + L5_3
      return L4_3
    else
      L4_3 = L8_1
      if L3_3 < L4_3 then
        L4_3 = L8_1
        L4_3 = A0_3 * L4_3
        L5_3 = L8_1
        L6_3 = A1_2
        L6_3 = L6_3[A1_3]
        L5_3 = L5_3 + L6_3
        L4_3 = L4_3 / L5_3
        return L4_3
      else
        return L3_3
      end
    end
  end
  
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

L1_1.revertColorLinearLayer = L20_1

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L2_2 = A1_2.wbTint
  L2_2 = -L2_2
  L3_2 = _G
  L3_2 = L3_2.tempMultiplier
  L3_2 = 255 / L3_2
  L2_2 = L2_2 / L3_2
  L3_2 = A1_2.wbTemp
  L3_2 = -L3_2
  L4_2 = _G
  L4_2 = L4_2.tempMultiplier
  L4_2 = 255 / L4_2
  L3_2 = L3_2 / L4_2
  L4_2 = nil
  L5_2 = 0.01
  L6_2 = nil
  L7_2 = A1_2.wbTint
  L7_2 = L7_2 * L5_2
  L8_2 = A1_2.wbTemp
  L8_2 = L8_2 * L5_2
  L9_2 = A1_2.wbTonality
  if L9_2 == "neutralDensity" then
    L9_2 = {}
    L10_2 = -L3_2
    L10_2 = L10_2 / 2
    L11_2 = -L2_2
    L11_2 = L11_2 / 2
    L10_2 = L10_2 + L11_2
    L11_2 = -L3_2
    L11_2 = L11_2 / 2
    L12_2 = L2_2 / 2
    L11_2 = L11_2 + L12_2
    L12_2 = L3_2 / 2
    L13_2 = -L2_2
    L13_2 = L13_2 / 2
    L12_2 = L12_2 + L13_2
    L9_2[1] = L10_2
    L9_2[2] = L11_2
    L9_2[3] = L12_2
    L4_2 = L9_2
    L9_2 = {}
    L10_2 = L8_2 * 0.5
    L10_2 = 1 - L10_2
    L10_2 = 1 / L10_2
    L11_2 = L7_2 * 0.5
    L11_2 = 1 - L11_2
    L11_2 = 1 / L11_2
    L10_2 = L10_2 * L11_2
    L11_2 = L8_2 * 0.5
    L11_2 = 1 - L11_2
    L11_2 = 1 / L11_2
    L12_2 = L7_2 * 0.5
    L12_2 = 1 - L12_2
    L11_2 = L11_2 * L12_2
    L12_2 = L8_2 * 0.5
    L12_2 = 1 - L12_2
    L13_2 = L7_2 * 0.5
    L13_2 = 1 - L13_2
    L13_2 = 1 / L13_2
    L12_2 = L12_2 * L13_2
    L9_2[1] = L10_2
    L9_2[2] = L11_2
    L9_2[3] = L12_2
    L6_2 = L9_2
  else
    L9_2 = A1_2.wbTonality
    if L9_2 == "subtractDensity" then
      L9_2 = {}
      L10_2 = -L3_2
      L11_2 = -L2_2
      L10_2 = L10_2 + L11_2
      L11_2 = -L3_2
      L12_2 = -L2_2
      L9_2[1] = L10_2
      L9_2[2] = L11_2
      L9_2[3] = L12_2
      L4_2 = L9_2
      L9_2 = {}
      L10_2 = 1 - L8_2
      L10_2 = 1 / L10_2
      L11_2 = 1 - L7_2
      L11_2 = 1 / L11_2
      L10_2 = L10_2 * L11_2
      L11_2 = 1 - L8_2
      L11_2 = 1 / L11_2
      L12_2 = 1 - L7_2
      L12_2 = 1 / L12_2
      L9_2[1] = L10_2
      L9_2[2] = L11_2
      L9_2[3] = L12_2
      L6_2 = L9_2
    else
      L9_2 = A1_2.wbTonality
      if L9_2 == "tempTintDensity" then
        L9_2 = {}
        L10_2 = -L2_2
        L10_2 = L10_2 / 2
        L11_2 = 0
        L12_2 = L2_2 / 2
        L12_2 = L3_2 - L12_2
        L9_2[1] = L10_2
        L9_2[2] = L11_2
        L9_2[3] = L12_2
        L4_2 = L9_2
      else
        L9_2 = {}
        L10_2 = 0
        L11_2 = L2_2
        L12_2 = L3_2
        L9_2[1] = L10_2
        L9_2[2] = L11_2
        L9_2[3] = L12_2
        L4_2 = L9_2
        L9_2 = {}
        L10_2 = 1
        L11_2 = 1 - L7_2
        L12_2 = 1 - L8_2
        L9_2[1] = L10_2
        L9_2[2] = L11_2
        L9_2[3] = L12_2
        L6_2 = L9_2
      end
    end
  end
  L9_2 = L9_1
  L9_2 = 1 - L9_2
  L10_2 = L18_1
  L11_2 = A0_2
  
  function L12_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
    L3_3 = L4_2
    L3_3 = L3_3[A1_3]
    L3_3 = A0_3 + L3_3
    L4_3 = nil
    L5_3 = nil
    L6_3 = nil
    L7_3 = A1_2
    L7_3 = L7_3.wbMethod
    if L7_3 == "linearFixed" then
      if 0 < A0_3 and A0_3 < 1 then
        L7_3 = L9_1
        if A0_3 > L7_3 then
          L7_3 = L4_2
          L7_3 = L7_3[A1_3]
          L8_3 = L9_2
          L8_3 = L8_3 * A0_3
          L9_3 = L4_2
          L9_3 = L9_3[A1_3]
          L9_3 = L9_3 * A0_3
          L8_3 = L8_3 - L9_3
          L9_3 = L4_2
          L9_3 = L9_3[A1_3]
          L10_3 = L9_1
          L9_3 = L9_3 * L10_3
          L8_3 = L8_3 + L9_3
          L9_3 = L9_2
          L8_3 = L8_3 / L9_3
          L6_3 = L7_3 + L8_3
          L7_3 = L12_1
          L8_3 = L6_3
          return L7_3(L8_3)
        else
          L7_3 = L8_1
          if A0_3 < L7_3 then
            L7_3 = L4_2
            L7_3 = L7_3[A1_3]
            L7_3 = L7_3 * A0_3
            L8_3 = L8_1
            L7_3 = L7_3 / L8_3
            L6_3 = A0_3 + L7_3
            L7_3 = L12_1
            L8_3 = L6_3
            return L7_3(L8_3)
          else
            L7_3 = L12_1
            L8_3 = L3_3
            return L7_3(L8_3)
          end
        end
      else
        L7_3 = L12_1
        L8_3 = A0_3
        return L7_3(L8_3)
      end
    else
      L7_3 = A1_2
      L7_3 = L7_3.wbMethod
      if L7_3 == "linearDynamic" then
        L7_3 = L12_1
        L8_3 = L3_3
        return L7_3(L8_3)
      else
        L7_3 = A1_2
        L7_3 = L7_3.wbMethod
        if L7_3 == "shadowWeighted" then
          L7_3 = L12_1
          L8_3 = L6_2
          L8_3 = L8_3[A1_3]
          L8_3 = 1 / L8_3
          L8_3 = A0_3 ^ L8_3
          return L7_3(L8_3)
        else
          L7_3 = A1_2
          L7_3 = L7_3.wbMethod
          if L7_3 == "highlightWeighted" then
            L7_3 = L12_1
            L8_3 = 1 - A0_3
            L9_3 = L6_2
            L9_3 = L9_3[A1_3]
            L8_3 = L8_3 ^ L9_3
            L8_3 = 1 - L8_3
            return L7_3(L8_3)
          else
            L7_3 = A1_2
            L7_3 = L7_3.wbMethod
            if L7_3 == "midtoneWeighted" then
              L7_3 = L12_1
              L8_3 = L6_2
              L8_3 = L8_3[A1_3]
              L8_3 = 1 / L8_3
              L8_3 = A0_3 ^ L8_3
              L9_3 = 1 - A0_3
              L10_3 = L6_2
              L10_3 = L10_3[A1_3]
              L9_3 = L9_3 ^ L10_3
              L9_3 = 1 - L9_3
              L8_3 = L8_3 + L9_3
              L8_3 = L8_3 / 2
              return L7_3(L8_3)
            end
          end
        end
      end
    end
  end
  
  L10_2 = L10_2(L11_2, L12_2)
  return L10_2
end

L1_1.colorLinearLayer = L20_1

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L3_2 = 0
  L4_2 = 0.2
  L5_2 = L18_1
  L6_2 = A0_2
  
  function L7_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3
    L3_3 = 0.2
    L4_3 = A1_2
    L4_3 = L4_3[A1_3]
    L4_3 = 1 - L4_3
    L4_3 = L4_3 / 4
    L4_3 = A0_3 - L4_3
    if 0 < A0_3 and A0_3 < 1 then
      if 1 <= L4_3 then
        L5_3 = 1
        return L5_3
      else
        L5_3 = 1 - L3_3
        if A0_3 > L5_3 then
          L5_3 = 1 - A0_3
          L5_3 = L5_3 / L3_3
          L6_3 = A1_2
          L6_3 = L6_3[A1_3]
          L6_3 = 1 - L6_3
          L6_3 = L6_3 / 4
          L6_3 = L6_3 * L5_3
          L6_3 = A0_3 - L6_3
          return L6_3
        elseif L4_3 <= 0 then
          L5_3 = 0
          return L5_3
        else
          return L4_3
        end
      end
    else
      return A0_3
    end
  end
  
  L5_2 = L5_2(L6_2, L7_2)
  return L5_2
end

L1_1.colorGammaLayer = L20_1

function L20_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L18_1
  L4_2 = A0_2
  
  function L5_2(A0_3, A1_3)
    local L2_3
    L2_3 = A1_2
    L2_3 = A0_3 ^ L2_3
    return L2_3
  end
  
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

L1_1.gammaLayer = L20_1

function L20_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L18_1
  L4_2 = A0_2
  
  function L5_2(A0_3, A1_3)
    local L2_3, L3_3, L4_3
    L2_3 = A1_2
    if L2_3 < 1 then
      L2_3 = 1 - A0_3
      L3_3 = A1_2
      L3_3 = 1 / L3_3
      L2_3 = L2_3 ^ L3_3
      L2_3 = 1 - L2_3
      return L2_3
    else
      L2_3 = A1_2
      if 1 < L2_3 then
        L2_3 = A1_2
        L2_3 = 2 - L2_3
        L3_3 = 0.4
        L4_3 = 1 - L2_3
        L4_3 = L4_3 * L3_3
        L4_3 = 1 - L4_3
        L4_3 = A0_3 * L4_3
        return L4_3
      else
        return A0_3
      end
    end
  end
  
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

L1_1.exposureLayer = L20_1

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = 0.5
  L3_2 = {}
  L4_2 = 0.2
  if 1 <= A1_2 then
    L5_2 = A1_2 * L4_2
    A1_2 = 1 + L5_2
    L5_2 = L18_1
    L6_2 = A0_2
    
    function L7_2(A0_3, A1_3)
      local L2_3, L3_3, L4_3, L5_3
      L2_3 = L14_1
      L3_3 = A1_2
      L4_3 = L2_2
      L5_3 = A0_3
      return L2_3(L3_3, L4_3, L5_3)
    end
    
    L5_2 = L5_2(L6_2, L7_2)
    L3_2 = L5_2
  elseif A1_2 <= -1 then
    L5_2 = math
    L5_2 = L5_2.abs
    L6_2 = A1_2
    L5_2 = L5_2(L6_2)
    L5_2 = L5_2 * L4_2
    L5_2 = L5_2 * 0.5
    A1_2 = 1 + L5_2
    L5_2 = L18_1
    L6_2 = A0_2
    
    function L7_2(A0_3, A1_3)
      local L2_3, L3_3, L4_3, L5_3
      L2_3 = L15_1
      L3_3 = A1_2
      L4_3 = L2_2
      L5_3 = A0_3
      return L2_3(L3_3, L4_3, L5_3)
    end
    
    L5_2 = L5_2(L6_2, L7_2)
    L3_2 = L5_2
  else
    return A0_2
  end
  return L3_2
end

L1_1.contrastLayer = L20_1

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L2_2 = {}
  L3_2 = 0.75
  L4_2 = 0.1
  L5_2 = 0.9
  L6_2 = math
  L6_2 = L6_2.abs
  L7_2 = A1_2
  L6_2 = L6_2(L7_2)
  L6_2 = L6_2 * L4_2
  L6_2 = 0.5 + L6_2
  if 1 <= A1_2 then
    L7_2 = A1_2 * L4_2
    L6_2 = 0.5 + L7_2
    L7_2 = L18_1
    L8_2 = A0_2
    
    function L9_2(A0_3, A1_3, A2_3)
      local L3_3, L4_3, L5_3, L6_3, L7_3
      L3_3 = L5_2
      L3_3 = 1 - L3_3
      if A0_3 <= L3_3 then
        return A0_3
      else
        L3_3 = L14_1
        L4_3 = L6_2
        L5_3 = L3_2
        L6_3 = 1 - A0_3
        L7_3 = L5_2
        L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3)
        L3_3 = 1 - L3_3
        return L3_3
      end
    end
    
    L7_2 = L7_2(L8_2, L9_2)
    L2_2 = L7_2
  elseif A1_2 <= -1 then
    L7_2 = L18_1
    L8_2 = A0_2
    
    function L9_2(A0_3, A1_3)
      local L2_3, L3_3, L4_3, L5_3, L6_3
      L2_3 = L5_2
      L2_3 = 1 - L2_3
      if A0_3 <= L2_3 then
        return A0_3
      else
        L2_3 = L15_1
        L3_3 = L6_2
        L4_3 = L3_2
        L5_3 = 1 - A0_3
        L6_3 = L5_2
        L2_3 = L2_3(L3_3, L4_3, L5_3, L6_3)
        L2_3 = 1 - L2_3
        return L2_3
      end
    end
    
    L7_2 = L7_2(L8_2, L9_2)
    L2_2 = L7_2
  else
    return A0_2
  end
  return L2_2
end

function L21_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L2_2 = 0.75
  L3_2 = 0.9
  L4_2 = 0.1
  L5_2 = math
  L5_2 = L5_2.abs
  L6_2 = A1_2
  L5_2 = L5_2(L6_2)
  L5_2 = L5_2 * L4_2
  L5_2 = 0.5 + L5_2
  L6_2 = {}
  if 0 < A1_2 then
    L7_2 = L18_1
    L8_2 = A0_2
    
    function L9_2(A0_3, A1_3)
      local L2_3, L3_3, L4_3, L5_3, L6_3
      L2_3 = L3_2
      if A0_3 >= L2_3 then
        return A0_3
      else
        L2_3 = L15_1
        L3_3 = L5_2
        L4_3 = L2_2
        L5_3 = A0_3
        L6_3 = L3_2
        return L2_3(L3_3, L4_3, L5_3, L6_3)
      end
    end
    
    L7_2 = L7_2(L8_2, L9_2)
    L6_2 = L7_2
  elseif A1_2 < 0 then
    L7_2 = L18_1
    L8_2 = A0_2
    
    function L9_2(A0_3, A1_3)
      local L2_3, L3_3, L4_3, L5_3, L6_3
      L2_3 = L3_2
      if A0_3 >= L2_3 then
        return A0_3
      else
        L2_3 = L14_1
        L3_3 = L5_2
        L4_3 = L2_2
        L5_3 = A0_3
        L6_3 = L3_2
        return L2_3(L3_3, L4_3, L5_3, L6_3)
      end
    end
    
    L7_2 = L7_2(L8_2, L9_2)
    L6_2 = L7_2
  else
    return A0_2
  end
  return L6_2
end

function L22_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L2_2 = A1_2.shadowRange
  L2_2 = -L2_2
  L2_2 = L2_2 + 1
  L2_2 = L2_2 * 0.33
  L2_2 = L2_2 + 5
  L3_2 = nil
  L4_2 = {}
  L5_2 = {}
  L6_2 = A1_2.shadowCyan
  L7_2 = A1_2.shadowTint
  L8_2 = A1_2.shadowTemp
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L6_2 = math
  L6_2 = L6_2.max
  L7_2 = unpack
  L8_2 = L5_2
  L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2 = L7_2(L8_2)
  L6_2 = L6_2(L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2)
  if 1 < L6_2 then
    L7_2 = 1
    L8_2 = #L5_2
    L9_2 = 1
    for L10_2 = L7_2, L8_2, L9_2 do
      L11_2 = L5_2[L10_2]
      L11_2 = L11_2 - L6_2
      L5_2[L10_2] = L11_2
    end
  end
  L7_2 = 0.75
  L8_2 = 0.1
  L9_2 = 0.5
  L10_2 = nil
  L11_2 = L18_1
  L12_2 = A0_2
  
  function L13_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
    L3_3 = A1_2
    L3_3 = L3_3.blacks
    if 1 <= L3_3 then
      L4_3 = L3_3 / 255
      L3_2 = L4_3
      if 0.9 <= A0_3 then
        return A0_3
      else
        L4_3 = L3_2
        L5_3 = math
        L5_3 = L5_3.exp
        L6_3 = -A0_3
        L7_3 = L2_2
        L6_3 = L6_3 * L7_3
        L5_3 = L5_3(L6_3)
        L4_3 = L4_3 * L5_3
        L4_3 = L4_3 + A0_3
        return L4_3
      end
    elseif L3_3 <= -1 then
      L4_3 = L9_2
      if A0_3 > L4_3 then
        return A0_3
      else
        L4_3 = math
        L4_3 = L4_3.abs
        L5_3 = L3_3
        L4_3 = L4_3(L5_3)
        L5_3 = L8_2
        L4_3 = L4_3 * L5_3
        L4_3 = 0.5 + L4_3
        L10_2 = L4_3
        L4_3 = L14_1
        L5_3 = L10_2
        L6_3 = L7_2
        L7_3 = A0_3
        L8_3 = L9_2
        return L4_3(L5_3, L6_3, L7_3, L8_3)
      end
    else
      return A0_3
    end
  end
  
  L11_2 = L11_2(L12_2, L13_2)
  L4_2 = L11_2
  return L4_2
end

function L23_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2
  L5_2 = A0_2 - A1_2
  L6_2 = A2_2 - A1_2
  L5_2 = L5_2 / L6_2
  L6_2 = A4_2 - A3_2
  L5_2 = L5_2 * L6_2
  L5_2 = L5_2 + A3_2
  return L5_2
end

function L24_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2
  if A3_2 == true then
    L4_2 = math
    L4_2 = L4_2.min
    L5_2 = A0_2
    L6_2 = A1_2
    L7_2 = A2_2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    if L4_2 < 0 then
      A0_2 = A0_2 - L4_2
      A1_2 = A1_2 - L4_2
      A2_2 = A2_2 - L4_2
    end
  else
    L4_2 = math
    L4_2 = L4_2.max
    L5_2 = A0_2
    L6_2 = A1_2
    L7_2 = A2_2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    if 0 < L4_2 then
      A0_2 = A0_2 - L4_2
      A1_2 = A1_2 - L4_2
      A2_2 = A2_2 - L4_2
    end
  end
  L4_2 = {}
  L5_2 = A0_2
  L6_2 = A1_2
  L7_2 = A2_2
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  return L4_2
end

function L25_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2
  L3_2 = 5 * A1_2
  L4_2 = 2 * A2_2
  L3_2 = L3_2 - L4_2
  L3_2 = L3_2 + 1.8
  if L3_2 < 1 then
    L3_2 = 1
  end
  if 2.5 < L3_2 then
    L3_2 = 2.5
  end
  L4_2 = A0_2
  if L4_2 < 0 then
    L4_2 = 0
  end
  if A2_2 >= L4_2 then
    L5_2 = L4_2 / A2_2
    L5_2 = L5_2 ^ L3_2
    L6_2 = A2_2 - A1_2
    L5_2 = L5_2 * L6_2
    L5_2 = A1_2 + L5_2
    return L5_2
  else
    return A0_2
  end
end

function L26_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L3_2 = L7_1
  L4_2 = L3_2
  L3_2 = L3_2.trace
  L5_2 = "Node Array:"
  L6_2 = L2_1
  L7_2 = A0_2
  L6_2 = L6_2(L7_2)
  L5_2 = L5_2 .. L6_2
  L3_2(L4_2, L5_2)
  L3_2 = {}
  L4_2 = 0.75
  L5_2 = 0.125
  L6_2 = A1_2.shadowRange
  L6_2 = 10 - L6_2
  L6_2 = L6_2 * 0.0444
  L6_2 = 0.9 - L6_2
  L7_2 = nil
  L8_2 = {}
  L9_2 = A1_2.shadowCyan
  L9_2 = -L9_2
  L10_2 = A1_2.shadowTint
  L10_2 = -L10_2
  L11_2 = A1_2.shadowTemp
  L11_2 = -L11_2
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L9_2 = L24_1
  L10_2 = A1_2.shadowCyan
  L10_2 = -L10_2
  L11_2 = A1_2.shadowTint
  L11_2 = -L11_2
  L12_2 = A1_2.shadowTemp
  L12_2 = -L12_2
  L13_2 = true
  L9_2 = L9_2(L10_2, L11_2, L12_2, L13_2)
  L10_2 = L18_1
  L11_2 = A0_2
  
  function L12_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
    L3_3 = L8_2
    L3_3 = L3_3[A1_3]
    L4_3 = math
    L4_3 = L4_3.abs
    L5_3 = L3_3
    L4_3 = L4_3(L5_3)
    L5_3 = A1_2
    L5_3 = L5_3.shadowRange
    L5_3 = 10 - L5_3
    L5_3 = L5_3 / 18
    L5_3 = 1 + L5_3
    L4_3 = L4_3 * L5_3
    L5_3 = L5_2
    L4_3 = L4_3 * L5_3
    L4_3 = 0.75 + L4_3
    L7_2 = L4_3
    L4_3 = A1_2
    L4_3 = L4_3.toningMethod
    if L4_3 == "vintageToning" then
      L4_3 = L9_2
      L4_3 = L4_3[A1_3]
      if L4_3 == 0 then
        return A0_3
      end
      L4_3 = L9_2
      L4_3 = L4_3[A1_3]
      L4_3 = L4_3 / 150
      L5_3 = L23_1
      L6_3 = A1_2
      L6_3 = L6_3.shadowRange
      L7_3 = 1
      L8_3 = 10
      L9_3 = 0.15
      L10_3 = 0.6
      L5_3 = L5_3(L6_3, L7_3, L8_3, L9_3, L10_3)
      L6_3 = math
      L6_3 = L6_3.min
      L7_3 = L5_3 + L4_3
      L8_3 = 0.99
      L6_3 = L6_3(L7_3, L8_3)
      L7_3 = L25_1
      L8_3 = A0_3
      L9_3 = L4_3
      L10_3 = L6_3
      L7_3 = L7_3(L8_3, L9_3, L10_3)
      return L7_3
    elseif L3_3 == 0 then
      return A0_3
    elseif 0 < L3_3 then
      L4_3 = L6_2
      if A0_3 >= L4_3 then
        return A0_3
      else
        L4_3 = L15_1
        L5_3 = L7_2
        L6_3 = L4_2
        L7_3 = A0_3
        L8_3 = L6_2
        return L4_3(L5_3, L6_3, L7_3, L8_3)
      end
    else
      L4_3 = L6_2
      if A0_3 >= L4_3 then
        return A0_3
      else
        L4_3 = L14_1
        L5_3 = L7_2
        L6_3 = L4_2
        L7_3 = A0_3
        L8_3 = L6_2
        return L4_3(L5_3, L6_3, L7_3, L8_3)
      end
    end
  end
  
  L10_2 = L10_2(L11_2, L12_2)
  L3_2 = L10_2
  return L3_2
end

function L27_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L3_2 = {}
  L4_2 = 0.75
  L5_2 = 0.125
  L6_2 = A1_2.highlightRange
  L6_2 = 10 - L6_2
  L6_2 = L6_2 * 0.0444
  L6_2 = 0.9 - L6_2
  L7_2 = nil
  L8_2 = {}
  L9_2 = A1_2.highlightCyan
  L9_2 = -L9_2
  L10_2 = A1_2.highlightTint
  L10_2 = -L10_2
  L11_2 = A1_2.highlightTemp
  L11_2 = -L11_2
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L9_2 = L24_1
  L10_2 = A1_2.highlightCyan
  L11_2 = A1_2.highlightTint
  L12_2 = A1_2.highlightTemp
  L13_2 = true
  L9_2 = L9_2(L10_2, L11_2, L12_2, L13_2)
  L10_2 = nil
  L11_2 = L18_1
  L12_2 = A0_2
  
  function L13_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
    L3_3 = A1_2
    L3_3 = L3_3.toningMethod
    if L3_3 == "vintageToning" then
      L3_3 = L9_2
      L3_3 = L3_3[A1_3]
      if L3_3 == 0 then
        return A0_3
      end
      L3_3 = L9_2
      L3_3 = L3_3[A1_3]
      L3_3 = L3_3 / 150
      L4_3 = L23_1
      L5_3 = A1_2
      L5_3 = L5_3.highlightRange
      L6_3 = 1
      L7_3 = 10
      L8_3 = 0.15
      L9_3 = 0.6
      L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3, L9_3)
      L5_3 = math
      L5_3 = L5_3.min
      L6_3 = L4_3 + L3_3
      L7_3 = 0.99
      L5_3 = L5_3(L6_3, L7_3)
      L6_3 = L25_1
      L7_3 = 1 - A0_3
      L8_3 = L3_3
      L9_3 = L5_3
      L6_3 = L6_3(L7_3, L8_3, L9_3)
      L6_3 = 1 - L6_3
      return L6_3
    else
      L3_3 = L8_2
      L3_3 = L3_3[A1_3]
      whiteColorSetting = L3_3
      L3_3 = math
      L3_3 = L3_3.abs
      L4_3 = whiteColorSetting
      L3_3 = L3_3(L4_3)
      L4_3 = A1_2
      L4_3 = L4_3.highlightRange
      L4_3 = 10 - L4_3
      L4_3 = L4_3 / 18
      L4_3 = 1 + L4_3
      L3_3 = L3_3 * L4_3
      L4_3 = L5_2
      L3_3 = L3_3 * L4_3
      L3_3 = 0.75 + L3_3
      L7_2 = L3_3
      L3_3 = whiteColorSetting
      if L3_3 == 0 then
        return A0_3
      else
        L3_3 = whiteColorSetting
        if 0 < L3_3 then
          L3_3 = L6_2
          L3_3 = 1 - L3_3
          if A0_3 <= L3_3 then
            return A0_3
          else
            L3_3 = L14_1
            L4_3 = L7_2
            L5_3 = L4_2
            L6_3 = 1 - A0_3
            L7_3 = L6_2
            L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3)
            L3_3 = 1 - L3_3
            return L3_3
          end
        else
          L3_3 = L6_2
          L3_3 = 1 - L3_3
          if A0_3 <= L3_3 then
            return A0_3
          else
            L3_3 = L15_1
            L4_3 = L7_2
            L5_3 = L4_2
            L6_3 = 1 - A0_3
            L7_3 = L6_2
            L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3)
            L3_3 = 1 - L3_3
            return L3_3
          end
        end
      end
    end
  end
  
  L11_2 = L11_2(L12_2, L13_2)
  L3_2 = L11_2
  return L3_2
end

function L28_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2
  L4_2 = A1_2.highlightRange
  L4_2 = -L4_2
  L4_2 = L4_2 + 1
  L4_2 = L4_2 * 0.33
  L4_2 = L4_2 + 5
  L5_2 = {}
  L6_2 = nil
  L7_2 = 0.75
  L8_2 = 0.1
  L9_2 = 0.5
  L10_2 = nil
  L11_2 = {}
  L12_2 = A1_2.highlightCyan
  L12_2 = -L12_2
  L13_2 = A1_2.highlightTint
  L13_2 = -L13_2
  L14_2 = A1_2.highlightTemp
  L14_2 = -L14_2
  L11_2[1] = L12_2
  L11_2[2] = L13_2
  L11_2[3] = L14_2
  L12_2 = math
  L12_2 = L12_2.max
  L13_2 = unpack
  L14_2 = L11_2
  L13_2, L14_2, L15_2, L16_2, L17_2 = L13_2(L14_2)
  L12_2 = L12_2(L13_2, L14_2, L15_2, L16_2, L17_2)
  if 1 < L12_2 then
    L13_2 = 1
    L14_2 = #L11_2
    L15_2 = 1
    for L16_2 = L13_2, L14_2, L15_2 do
      L17_2 = L11_2[L16_2]
      L17_2 = L17_2 - L12_2
      L11_2[L16_2] = L17_2
    end
  end
  L13_2 = A2_2.whites
  L14_2 = A1_2.highlights
  if L14_2 < -5 then
    L14_2 = A1_2.highlights
    L14_2 = L14_2 / 5
    L13_2 = L13_2 + L14_2
  end
  L14_2 = L18_1
  L15_2 = A0_2
  
  function L16_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
    L3_3 = L13_2
    if L3_3 <= -1 then
      if A0_3 < 0.1 then
        return A0_3
      else
        L4_3 = -L3_3
        L4_3 = L4_3 / 255
        L6_2 = L4_3
        L4_3 = L6_2
        L5_3 = math
        L5_3 = L5_3.exp
        L6_3 = 1 - A0_3
        L6_3 = -L6_3
        L7_3 = L4_2
        L6_3 = L6_3 * L7_3
        L5_3 = L5_3(L6_3)
        L4_3 = L4_3 * L5_3
        L5_3 = 1 - A0_3
        L4_3 = L4_3 + L5_3
        L4_3 = 1 - L4_3
        return L4_3
      end
    elseif 1 <= L3_3 then
      L4_3 = L9_2
      L4_3 = 1 - L4_3
      if A0_3 < L4_3 then
        return A0_3
      else
        L4_3 = math
        L4_3 = L4_3.abs
        L5_3 = L3_3
        L4_3 = L4_3(L5_3)
        L5_3 = L8_2
        L4_3 = L4_3 * L5_3
        L4_3 = 0.5 + L4_3
        L10_2 = L4_3
        L4_3 = L14_1
        L5_3 = L10_2
        L6_3 = L7_2
        L7_3 = 1 - A0_3
        L8_3 = L9_2
        L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3)
        L4_3 = 1 - L4_3
        return L4_3
      end
    else
      return A0_3
    end
  end
  
  L14_2 = L14_2(L15_2, L16_2)
  L5_2 = L14_2
  return L5_2
end

function L29_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L18_1
  L4_2 = A0_2
  
  function L5_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3
    L3_3 = A1_2
    L3_3 = L3_3[A1_3]
    if L3_3 == 0 then
      L3_3 = A2_2
      L3_3 = L3_3[A1_3]
      if L3_3 == 0 then
        return A0_3
    end
    else
      L3_3 = A1_2
      L3_3 = L3_3[A1_3]
      L3_3 = 255 - L3_3
      L3_3 = L3_3 / 255
      L4_3 = A2_2
      L4_3 = L4_3[A1_3]
      L4_3 = L4_3 / 255
      L5_3 = L3_3 - L4_3
      L5_3 = A0_3 * L5_3
      L5_3 = L5_3 + L4_3
      return L5_3
    end
  end
  
  L3_2 = L3_2(L4_2, L5_2)
  return L3_2
end

function L30_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = A0_2[1]
  L1_2 = #L1_2
  L2_2 = L18_1
  L3_2 = A0_2
  
  function L4_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3
    L3_3 = A0_2
    L3_3 = L3_3[A1_3]
    L4_3 = L1_2
    L4_3 = L4_3 + 1
    L4_3 = L4_3 - A2_3
    L3_3 = L3_3[L4_3]
    return L3_3
  end
  
  L2_2 = L2_2(L3_2, L4_2)
  returnArray = L2_2
  L2_2 = returnArray
  return L2_2
end

L1_1.invert = L30_1

function L30_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = A0_2[1]
  L1_2 = #L1_2
  L2_2 = nil
  L3_2 = nil
  L4_2 = nil
  L5_2 = L18_1
  L6_2 = A0_2
  
  function L7_2(A0_3, A1_3, A2_3)
    local L3_3, L4_3, L5_3, L6_3, L7_3
    L4_3 = A0_2
    L4_3 = L4_3[A1_3]
    L4_3 = L4_3[1]
    L2_2 = L4_3
    L4_3 = A0_2
    L4_3 = L4_3[A1_3]
    L5_3 = L1_2
    L4_3 = L4_3[L5_3]
    L3_2 = L4_3
    if A2_3 ~= 1 then
      L4_3 = L1_2
      if A2_3 ~= L4_3 then
        goto lbl_17
      end
    end
    L3_3 = A0_3
    goto lbl_27
    ::lbl_17::
    L4_3 = L2_2
    L5_3 = L3_2
    L6_3 = A0_2
    L6_3 = L6_3[A1_3]
    L7_3 = L1_2
    L7_3 = L7_3 - A2_3
    L7_3 = L7_3 + 1
    L6_3 = L6_3[L7_3]
    L5_3 = L5_3 - L6_3
    L3_3 = L4_3 + L5_3
    ::lbl_27::
    return L3_3
  end
  
  L5_2 = L5_2(L6_2, L7_2)
  returnArray = L5_2
  L5_2 = returnArray
  return L5_2
end

L1_1.mirror = L30_1

function L30_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L5_2 = A2_2.origin
  L6_2 = A2_2.origin
  L6_2 = L6_2 / 2
  L7_2 = A1_2.origin
  L8_2 = A1_2.origin
  L8_2 = L8_2 / 2
  L9_2 = A0_2.wbTemp
  L9_2 = L9_2 / L5_2
  L9_2 = A3_2 * L9_2
  L10_2 = A0_2.wbTemp
  L10_2 = L10_2 / L6_2
  L10_2 = A4_2 * L10_2
  L11_2 = A0_2.wbTint
  L11_2 = L11_2 / L7_2
  L11_2 = A3_2 * L11_2
  L12_2 = A0_2.wbTint
  L12_2 = L12_2 / L8_2
  L12_2 = A4_2 * L12_2
  L13_2 = L9_2
  L14_2 = L10_2
  L15_2 = L11_2
  L16_2 = L12_2
  return L13_2, L14_2, L15_2, L16_2
end

function L31_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2
  L4_2 = A0_2[1]
  L4_2 = L4_2.whitePointOrigin
  L4_2 = L4_2 + A2_2
  L5_2 = A0_2[1]
  L5_2 = L5_2.blackPointOrigin
  L5_2 = L5_2 - A3_2
  L6_2 = nil
  L7_2 = nil
  L8_2 = nil
  L9_2 = {}
  L10_2 = A0_2[1]
  L10_2 = L10_2.blackPointOrigin
  L11_2 = A0_2[1]
  L11_2 = L11_2.whitePointOrigin
  L10_2 = L10_2 - L11_2
  L9_2.origin = L10_2
  L10_2 = A0_2[1]
  L10_2 = L10_2.blackPointOrigin
  L10_2 = L10_2 - L4_2
  L9_2.afterWhite = L10_2
  L10_2 = A0_2[1]
  L10_2 = L10_2.whitePointOrigin
  L10_2 = L5_2 - L10_2
  L9_2.afterBlack = L10_2
  L10_2 = L5_2 - L4_2
  L9_2.new = L10_2
  L6_2 = L9_2
  L9_2 = {}
  L10_2 = A0_2[2]
  L10_2 = L10_2.blackPointOrigin
  L11_2 = A0_2[2]
  L11_2 = L11_2.whitePointOrigin
  L10_2 = L10_2 - L11_2
  L9_2.origin = L10_2
  L7_2 = L9_2
  L9_2 = {}
  L10_2 = A0_2[3]
  L10_2 = L10_2.blackPointOrigin
  L11_2 = A0_2[3]
  L11_2 = L11_2.whitePointOrigin
  L10_2 = L10_2 - L11_2
  L9_2.origin = L10_2
  L8_2 = L9_2
  L9_2 = L30_1
  L10_2 = A1_2
  L11_2 = L7_2
  L12_2 = L8_2
  L13_2 = A2_2
  L14_2 = A3_2
  L9_2, L10_2, L11_2, L12_2 = L9_2(L10_2, L11_2, L12_2, L13_2, L14_2)
  L13_2 = nil
  L14_2 = nil
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = L6_2.afterWhite
  L19_2 = L6_2.origin
  L13_2 = L18_2 / L19_2
  L18_2 = A0_2[2]
  L18_2 = L18_2.blackPointOrigin
  L19_2 = L7_2.origin
  L19_2 = L13_2 * L19_2
  L18_2 = L18_2 - L19_2
  L14_2 = L18_2 - L11_2
  L18_2 = A0_2[3]
  L18_2 = L18_2.blackPointOrigin
  L19_2 = L8_2.origin
  L19_2 = L13_2 * L19_2
  L18_2 = L18_2 - L19_2
  L15_2 = L18_2 - L9_2
  L18_2 = nil
  L19_2 = nil
  L20_2 = nil
  L21_2 = L6_2.afterBlack
  L22_2 = L6_2.origin
  L18_2 = L21_2 / L22_2
  L21_2 = A0_2[2]
  L21_2 = L21_2.whitePointOrigin
  L22_2 = L7_2.origin
  L22_2 = L18_2 * L22_2
  L21_2 = L21_2 + L22_2
  L19_2 = L21_2 - L12_2
  L21_2 = A0_2[3]
  L21_2 = L21_2.whitePointOrigin
  L22_2 = L8_2.origin
  L22_2 = L18_2 * L22_2
  L21_2 = L21_2 + L22_2
  L20_2 = L21_2 - L10_2
  L21_2 = L4_2
  L22_2 = L14_2
  L23_2 = L15_2
  L24_2 = L5_2
  L25_2 = L19_2
  L26_2 = L20_2
  return L21_2, L22_2, L23_2, L24_2, L25_2, L26_2
end

function L32_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2
  L4_2 = A1_2.toneProfile
  L5_2 = L5_1
  L5_2 = L5_2.toneProfiles_v3
  L5_2 = L5_2[L4_2]
  L6_2 = L5_2.defaultSoftHigh
  L7_2 = L5_2.defaultSoftLow
  L8_2 = A2_2 or L8_2
  if not A2_2 then
    L8_2 = 0
  end
  L9_2 = A3_2 or L9_2
  if not A3_2 then
    L9_2 = 0
  end
  L10_2 = nil
  L11_2 = nil
  L12_2 = A1_2.imageType
  if L12_2 ~= "positive" then
    L12_2 = A1_2.imageType
    if L12_2 ~= "editOnly" then
      goto lbl_27
    end
  end
  L12_2 = A1_2.softLow
  L12_2 = L12_2 + L7_2
  L10_2 = L12_2 + L9_2
  L12_2 = A1_2.softHigh
  L12_2 = L12_2 + L6_2
  L11_2 = L12_2 + L8_2
  goto lbl_33
  ::lbl_27::
  L12_2 = A1_2.softHigh
  L12_2 = L12_2 + L6_2
  L10_2 = L12_2 + L8_2
  L12_2 = A1_2.softLow
  L12_2 = L12_2 + L7_2
  L11_2 = L12_2 + L9_2
  ::lbl_33::
  L12_2 = L31_1
  L13_2 = A0_2
  L14_2 = A1_2
  L15_2 = L10_2
  L16_2 = L11_2
  L12_2, L13_2, L14_2, L15_2, L16_2, L17_2 = L12_2(L13_2, L14_2, L15_2, L16_2)
  L18_2 = {}
  L19_2 = L12_2
  L20_2 = L13_2
  L21_2 = L14_2
  L18_2[1] = L19_2
  L18_2[2] = L20_2
  L18_2[3] = L21_2
  L19_2 = {}
  L20_2 = 0
  L21_2 = 0
  L22_2 = 0
  L19_2[1] = L20_2
  L19_2[2] = L21_2
  L19_2[3] = L22_2
  L20_2 = {}
  L21_2 = L15_2
  L22_2 = L16_2
  L23_2 = L17_2
  L20_2[1] = L21_2
  L20_2[2] = L22_2
  L20_2[3] = L23_2
  L21_2 = {}
  L22_2 = 0
  L23_2 = 0
  L24_2 = 0
  L21_2[1] = L22_2
  L21_2[2] = L23_2
  L21_2[3] = L24_2
  L22_2 = 1
  L23_2 = #L18_2
  L24_2 = 1
  for L25_2 = L22_2, L23_2, L24_2 do
    L26_2 = L18_2[L25_2]
    if L26_2 < 0 then
      L26_2 = L3_1
      L26_2 = L26_2.calculate_slope
      L27_2 = L20_2[L25_2]
      L28_2 = 0
      L29_2 = L18_2[L25_2]
      L30_2 = 255
      L26_2 = L26_2(L27_2, L28_2, L29_2, L30_2)
      L27_2 = L3_1
      L27_2 = L27_2.calculate_y
      L28_2 = L26_2
      L29_2 = L20_2[L25_2]
      L30_2 = 0
      L31_2 = 0
      L27_2 = L27_2(L28_2, L29_2, L30_2, L31_2)
      L28_2 = 255 - L27_2
      L19_2[L25_2] = L28_2
      L18_2[L25_2] = 0
    end
  end
  L22_2 = 1
  L23_2 = #L20_2
  L24_2 = 1
  for L25_2 = L22_2, L23_2, L24_2 do
    L26_2 = L20_2[L25_2]
    if 255 < L26_2 then
      L26_2 = L3_1
      L26_2 = L26_2.calculate_slope
      L27_2 = L20_2[L25_2]
      L28_2 = 0
      L29_2 = L18_2[L25_2]
      L30_2 = 255
      L26_2 = L26_2(L27_2, L28_2, L29_2, L30_2)
      L27_2 = L3_1
      L27_2 = L27_2.calculate_y
      L28_2 = L26_2
      L29_2 = L20_2[L25_2]
      L30_2 = 0
      L31_2 = 255
      L27_2 = L27_2(L28_2, L29_2, L30_2, L31_2)
      L21_2[L25_2] = L27_2
      L20_2[L25_2] = 255
      L20_2[L25_2] = 255
    end
  end
  L22_2 = L20_2[1]
  L23_2 = L18_2[1]
  L22_2 = L22_2 - L23_2
  rWidth = L22_2
  L22_2 = L20_2[2]
  L23_2 = L18_2[2]
  L22_2 = L22_2 - L23_2
  gWidth = L22_2
  L22_2 = L20_2[3]
  L23_2 = L18_2[3]
  L22_2 = L22_2 - L23_2
  bWidth = L22_2
  L22_2 = L18_2[1]
  L23_2 = L18_2[2]
  L24_2 = L18_2[3]
  L25_2 = L20_2[1]
  L26_2 = L20_2[2]
  L27_2 = L20_2[3]
  L28_2 = rWidth
  L29_2 = gWidth
  L30_2 = bWidth
  L31_2 = L19_2[1]
  L32_2 = L19_2[2]
  L33_2 = L19_2[3]
  L34_2 = L21_2[1]
  L35_2 = L21_2[2]
  L36_2 = L21_2[3]
  return L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2
end

function L33_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2
  L10_2 = A1_2.toneProfile
  L11_2 = L5_1
  L11_2 = L11_2.toneProfiles
  L11_2 = L11_2[L10_2]
  L12_2 = L11_2.defaultSoftHigh
  L13_2 = L11_2.defaultSoftLow
  L14_2 = L5_1
  L14_2 = L14_2.cameraScan
  L14_2 = L14_2.colorModels
  L15_2 = A1_2.colorModel
  L14_2 = L14_2[L15_2]
  L14_2 = L14_2.negativeLabSettings
  L15_2 = A1_2.imageType
  if L15_2 ~= "positive" then
    L15_2 = A1_2.imageType
    if L15_2 ~= "editOnly" then
      goto lbl_50
    end
  end
  L15_2 = A0_2[1]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 + L16_2
  L4_2 = L15_2 + L13_2
  L15_2 = A0_2[2]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 + L16_2
  L5_2 = L15_2 + L13_2
  L15_2 = A0_2[3]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 + L16_2
  L6_2 = L15_2 + L13_2
  L15_2 = A0_2[1]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 - L16_2
  L7_2 = L15_2 - L12_2
  L15_2 = A0_2[2]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 - L16_2
  L8_2 = L15_2 - L12_2
  L15_2 = A0_2[3]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 - L16_2
  L9_2 = L15_2 - L12_2
  goto lbl_80
  ::lbl_50::
  L15_2 = A0_2[1]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 + L16_2
  L4_2 = L15_2 + L12_2
  L15_2 = A0_2[2]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 + L16_2
  L5_2 = L15_2 + L12_2
  L15_2 = A0_2[3]
  L15_2 = L15_2.whitePointOrigin
  L16_2 = A1_2.softHigh
  L15_2 = L15_2 + L16_2
  L6_2 = L15_2 + L12_2
  L15_2 = A0_2[1]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 - L16_2
  L7_2 = L15_2 - L13_2
  L15_2 = A0_2[2]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 - L16_2
  L8_2 = L15_2 - L13_2
  L15_2 = A0_2[3]
  L15_2 = L15_2.blackPointOrigin
  L16_2 = A1_2.softLow
  L15_2 = L15_2 - L16_2
  L9_2 = L15_2 - L13_2
  ::lbl_80::
  L15_2 = A2_2 or L15_2
  if not A2_2 then
    L15_2 = 0
  end
  L16_2 = A3_2 or L16_2
  if not A3_2 then
    L16_2 = 0
  end
  if L15_2 ~= 0 or L16_2 ~= 0 then
    L17_2 = {}
    L18_2 = {}
    L18_2.whitePointOrigin = L4_2
    L18_2.blackPointOrigin = L7_2
    L19_2 = {}
    L19_2.whitePointOrigin = L5_2
    L19_2.blackPointOrigin = L8_2
    L20_2 = {}
    L20_2.whitePointOrigin = L6_2
    L20_2.blackPointOrigin = L9_2
    L17_2[1] = L18_2
    L17_2[2] = L19_2
    L17_2[3] = L20_2
    L18_2 = L31_1
    L19_2 = A0_2
    L20_2 = A1_2
    L21_2 = L15_2
    L22_2 = L16_2
    L18_2, L19_2, L20_2, L21_2, L22_2, L23_2 = L18_2(L19_2, L20_2, L21_2, L22_2)
    L9_2 = L23_2
    L8_2 = L22_2
    L7_2 = L21_2
    L6_2 = L20_2
    L5_2 = L19_2
    L4_2 = L18_2
  end
  L17_2 = {}
  L18_2 = L4_2
  L19_2 = L5_2
  L20_2 = L6_2
  L17_2[1] = L18_2
  L17_2[2] = L19_2
  L17_2[3] = L20_2
  L18_2 = {}
  L19_2 = 0
  L20_2 = 0
  L21_2 = 0
  L18_2[1] = L19_2
  L18_2[2] = L20_2
  L18_2[3] = L21_2
  L19_2 = {}
  L20_2 = L7_2
  L21_2 = L8_2
  L22_2 = L9_2
  L19_2[1] = L20_2
  L19_2[2] = L21_2
  L19_2[3] = L22_2
  L20_2 = {}
  L21_2 = 0
  L22_2 = 0
  L23_2 = 0
  L20_2[1] = L21_2
  L20_2[2] = L22_2
  L20_2[3] = L23_2
  L21_2 = 1
  L22_2 = #L17_2
  L23_2 = 1
  for L24_2 = L21_2, L22_2, L23_2 do
    L25_2 = L17_2[L24_2]
    if L25_2 < 0 then
      L25_2 = L17_2[L24_2]
      L25_2 = -L25_2
      L18_2[L24_2] = L25_2
      L17_2[L24_2] = 0
    end
  end
  L21_2 = 1
  L22_2 = #L19_2
  L23_2 = 1
  for L24_2 = L21_2, L22_2, L23_2 do
    L25_2 = L19_2[L24_2]
    if 255 < L25_2 then
      L25_2 = L19_2[L24_2]
      L25_2 = L25_2 - 255
      L20_2[L24_2] = L25_2
      L19_2[L24_2] = 255
    end
  end
  L21_2 = L19_2[1]
  L22_2 = L17_2[1]
  L21_2 = L21_2 - L22_2
  rWidth = L21_2
  L21_2 = L19_2[2]
  L22_2 = L17_2[2]
  L21_2 = L21_2 - L22_2
  gWidth = L21_2
  L21_2 = L19_2[3]
  L22_2 = L17_2[3]
  L21_2 = L21_2 - L22_2
  bWidth = L21_2
  L21_2 = L17_2[1]
  L22_2 = L17_2[2]
  L23_2 = L17_2[3]
  L24_2 = L19_2[1]
  L25_2 = L19_2[2]
  L26_2 = L19_2[3]
  L27_2 = rWidth
  L28_2 = gWidth
  L29_2 = bWidth
  L30_2 = L18_2[1]
  L31_2 = L18_2[2]
  L32_2 = L18_2[3]
  L33_2 = L20_2[1]
  L34_2 = L20_2[2]
  L35_2 = L20_2[3]
  return L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2
end

function L34_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2
  L5_2 = 6
  L6_2 = nil
  L7_2 = nil
  L8_2 = A1_2.imageType
  if L8_2 ~= "positive" then
    L8_2 = A1_2.imageType
    if L8_2 ~= "editOnly" then
      goto lbl_12
    end
  end
  L6_2 = A1_2.softShadows
  L7_2 = A1_2.softHighlights
  goto lbl_14
  ::lbl_12::
  L6_2 = A1_2.softHighlights
  L7_2 = A1_2.softShadows
  ::lbl_14::
  L8_2 = nil
  L9_2 = nil
  L10_2 = nil
  L11_2 = nil
  L12_2 = nil
  L13_2 = nil
  L14_2 = nil
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = nil
  L19_2 = nil
  L20_2 = nil
  L21_2 = nil
  L22_2 = nil
  L23_2 = A1_2.clipMethod
  if L23_2 ~= nil then
    L23_2 = A1_2.clipMethod
    if L23_2 == "colorProtectedClipping" then
      L23_2 = isMonochrome
      if L23_2 ~= true then
        L23_2 = L32_1
        L24_2 = A0_2
        L25_2 = A1_2
        L26_2 = A3_2
        L27_2 = A4_2
        L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2 = L23_2(L24_2, L25_2, L26_2, L27_2)
        L22_2 = L37_2
        L21_2 = L36_2
        L20_2 = L35_2
        L19_2 = L34_2
        L18_2 = L33_2
        L17_2 = L32_2
        L16_2 = L31_2
        L15_2 = L30_2
        L14_2 = L29_2
        L13_2 = L28_2
        L12_2 = L27_2
        L11_2 = L26_2
        L10_2 = L25_2
        L9_2 = L24_2
        L8_2 = L23_2
    end
  end
  else
    L23_2 = L33_1
    L24_2 = A0_2
    L25_2 = A1_2
    L26_2 = A3_2
    L27_2 = A4_2
    L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2 = L23_2(L24_2, L25_2, L26_2, L27_2)
    L22_2 = L37_2
    L21_2 = L36_2
    L20_2 = L35_2
    L19_2 = L34_2
    L18_2 = L33_2
    L17_2 = L32_2
    L16_2 = L31_2
    L15_2 = L30_2
    L14_2 = L29_2
    L13_2 = L28_2
    L12_2 = L27_2
    L11_2 = L26_2
    L10_2 = L25_2
    L9_2 = L24_2
    L8_2 = L23_2
  end
  L23_2 = {}
  L24_2 = {}
  L25_2 = {}
  L26_2 = A2_2[1]
  L26_2 = #L26_2
  L27_2 = 1
  L28_2 = L26_2
  L29_2 = 1
  for L30_2 = L27_2, L28_2, L29_2 do
    L31_2 = A2_2[1]
    L31_2 = L31_2[L30_2]
    L31_2 = L31_2 * L14_2
    L31_2 = L8_2 + L31_2
    L23_2[L30_2] = L31_2
    L31_2 = A2_2[2]
    L31_2 = L31_2[L30_2]
    L31_2 = L31_2 * L15_2
    L31_2 = L9_2 + L31_2
    L24_2[L30_2] = L31_2
    L31_2 = A2_2[3]
    L31_2 = L31_2[L30_2]
    L31_2 = L31_2 * L16_2
    L31_2 = L10_2 + L31_2
    L25_2[L30_2] = L31_2
    if L30_2 == 1 and L6_2 == true then
      L31_2 = L23_2[1]
      if L5_2 < L31_2 then
        L31_2 = L24_2[1]
        if L5_2 < L31_2 then
          L31_2 = L25_2[1]
          if L5_2 < L31_2 then
            L31_2 = L23_2[1]
            L31_2 = L31_2 - L5_2
            L23_2[1] = L31_2
            L31_2 = L24_2[1]
            L31_2 = L31_2 - L5_2
            L24_2[1] = L31_2
            L31_2 = L25_2[1]
            L31_2 = L31_2 - L5_2
            L25_2[1] = L31_2
          end
        end
      end
    end
    if L30_2 == L26_2 and L7_2 == true then
      L31_2 = 255 - L5_2
      L32_2 = L23_2[L30_2]
      if L31_2 > L32_2 then
        L32_2 = L24_2[L30_2]
        if L31_2 > L32_2 then
          L32_2 = L25_2[L30_2]
          if L31_2 > L32_2 then
            L32_2 = L23_2[L30_2]
            L32_2 = L32_2 + L5_2
            L23_2[L30_2] = L32_2
            L32_2 = L24_2[L30_2]
            L32_2 = L32_2 + L5_2
            L24_2[L30_2] = L32_2
            L32_2 = L25_2[L30_2]
            L32_2 = L32_2 + L5_2
            L25_2[L30_2] = L32_2
          end
        end
      end
    end
  end
  L27_2 = {}
  L28_2 = L23_2
  L29_2 = L24_2
  L30_2 = L25_2
  L27_2[1] = L28_2
  L27_2[2] = L29_2
  L27_2[3] = L30_2
  return L27_2
end

function L35_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L5_2 = L7_1
  L6_2 = L5_2
  L5_2 = L5_2.trace
  L7_2 = "in makeMaskCurve"
  L5_2(L6_2, L7_2)
  L5_2 = {}
  L6_2 = A2_2[1]
  L6_2 = L6_2.settingName
  L7_2 = {}
  L5_2[L6_2] = L7_2
  L6_2 = A2_2[2]
  L6_2 = L6_2.settingName
  L7_2 = {}
  L5_2[L6_2] = L7_2
  L6_2 = A2_2[3]
  L6_2 = L6_2.settingName
  L7_2 = {}
  L5_2[L6_2] = L7_2
  L6_2 = L7_1
  L7_2 = L6_2
  L6_2 = L6_2.trace
  L8_2 = "channelOrigins: "
  L9_2 = L2_1
  L10_2 = A2_2
  L9_2 = L9_2(L10_2)
  L8_2 = L8_2 .. L9_2
  L6_2(L7_2, L8_2)
  L6_2 = 1
  L7_2 = #A2_2
  L8_2 = 1
  for L9_2 = L6_2, L7_2, L8_2 do
    L10_2 = L3_1
    L10_2 = L10_2.round
    L11_2 = A1_2[L9_2]
    L11_2 = L11_2[1]
    L10_2 = L10_2(L11_2)
    L11_2 = L3_1
    L11_2 = L11_2.round
    L12_2 = A1_2[L9_2]
    L13_2 = A1_2[L9_2]
    L13_2 = #L13_2
    L12_2 = L12_2[L13_2]
    L11_2 = L11_2(L12_2)
    L12_2 = table
    L12_2 = L12_2.insert
    L13_2 = A2_2[L9_2]
    L13_2 = L13_2.settingName
    L13_2 = L5_2[L13_2]
    L14_2 = L10_2
    L12_2(L13_2, L14_2)
    L12_2 = table
    L12_2 = L12_2.insert
    L13_2 = A2_2[L9_2]
    L13_2 = L13_2.settingName
    L13_2 = L5_2[L13_2]
    L14_2 = 255
    L12_2(L13_2, L14_2)
    L12_2 = table
    L12_2 = L12_2.insert
    L13_2 = A2_2[L9_2]
    L13_2 = L13_2.settingName
    L13_2 = L5_2[L13_2]
    L14_2 = L11_2
    L12_2(L13_2, L14_2)
    L12_2 = table
    L12_2 = L12_2.insert
    L13_2 = A2_2[L9_2]
    L13_2 = L13_2.settingName
    L13_2 = L5_2[L13_2]
    L14_2 = 0
    L12_2(L13_2, L14_2)
  end
  L6_2 = L17_1
  L7_2 = A0_2[1]
  L7_2 = #L7_2
  L6_2 = L6_2(L7_2)
  L7_2 = L7_1
  L8_2 = L7_2
  L7_2 = L7_2.trace
  L9_2 = "layerMaskXNodes: "
  L10_2 = L2_1
  L11_2 = L6_2
  L10_2 = L10_2(L11_2)
  L9_2 = L9_2 .. L10_2
  L7_2(L8_2, L9_2)
  L7_2 = {}
  L8_2 = {}
  L9_2 = {}
  L10_2 = {}
  L7_2[1] = L8_2
  L7_2[2] = L9_2
  L7_2[3] = L10_2
  L8_2 = 1
  L9_2 = #A2_2
  L10_2 = 1
  for L11_2 = L8_2, L9_2, L10_2 do
    L12_2 = 1
    L13_2 = A0_2[L11_2]
    L13_2 = #L13_2
    L14_2 = 1
    for L15_2 = L12_2, L13_2, L14_2 do
      L16_2 = table
      L16_2 = L16_2.insert
      L17_2 = L7_2[L11_2]
      L18_2 = L3_1
      L18_2 = L18_2.round
      L19_2 = L6_2[L11_2]
      L19_2 = L19_2[L15_2]
      L19_2 = L19_2 * 255
      L18_2, L19_2 = L18_2(L19_2)
      L16_2(L17_2, L18_2, L19_2)
      L16_2 = table
      L16_2 = L16_2.insert
      L17_2 = L7_2[L11_2]
      L18_2 = L3_1
      L18_2 = L18_2.round
      L19_2 = A0_2[L11_2]
      L19_2 = L19_2[L15_2]
      L19_2 = L19_2 * 255
      L18_2, L19_2 = L18_2(L19_2)
      L16_2(L17_2, L18_2, L19_2)
    end
  end
  
  function L8_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
    L1_3 = {}
    L2_3 = 1
    L3_3 = #A0_3
    L4_3 = 2
    for L5_3 = L2_3, L3_3, L4_3 do
      L6_3 = tostring
      L7_3 = A0_3[L5_3]
      L6_3 = L6_3(L7_3)
      L7_3 = ","
      L8_3 = tostring
      L9_3 = L5_3 + 1
      L9_3 = A0_3[L9_3]
      L8_3 = L8_3(L9_3)
      L6_3 = L6_3 .. L7_3 .. L8_3
      L7_3 = table
      L7_3 = L7_3.insert
      L8_3 = L1_3
      L9_3 = L6_3
      L7_3(L8_3, L9_3)
    end
    return L1_3
  end
  
  L9_2 = L8_2
  L10_2 = L7_2[1]
  L9_2 = L9_2(L10_2)
  L10_2 = L8_2
  L11_2 = L7_2[2]
  L10_2 = L10_2(L11_2)
  L11_2 = L8_2
  L12_2 = L7_2[3]
  L11_2 = L11_2(L12_2)
  L12_2 = {}
  L13_2 = _G
  L13_2 = L13_2.activePhotosMeta
  L13_2 = L13_2[A4_2]
  L13_2 = L13_2.activeMasks
  L14_2 = L7_1
  L15_2 = L14_2
  L14_2 = L14_2.trace
  L16_2 = "activeMasks: "
  L17_2 = L2_1
  L18_2 = L13_2
  L17_2 = L17_2(L18_2)
  L16_2 = L16_2 .. L17_2
  L14_2(L15_2, L16_2)
  L14_2 = {}
  L14_2.BlueCurve = L11_2
  L14_2.CorrectionActive = true
  L14_2.CorrectionAmount = 1
  L14_2.CorrectionID = "175060B0-2F91-4655-8F1B-10BAD02AF28E"
  L15_2 = {}
  L16_2 = {}
  L17_2 = {}
  L17_2.Invert = false
  L17_2.LumRange = "0.000000 0.000000 1.000000 1.000000"
  L17_2.SampleType = 0
  L17_2.Type = 2
  L17_2.Version = 3
  L16_2.CorrectionRangeMask = L17_2
  L16_2.MaskActive = true
  L16_2.MaskBlendMode = 0
  L16_2.MaskID = "32E0A04D-E3CA-45E4-A10B-6BC10D25D037"
  L16_2.MaskInverted = false
  L16_2.MaskName = "Luminance Range 1"
  L16_2.MaskSyncID = "8C2E547C986B4CD6B4C205DCA7885E28"
  L16_2.MaskValue = 1
  L16_2.What = "Mask/RangeMask"
  L15_2[1] = L16_2
  L14_2.CorrectionMasks = L15_2
  L14_2.CorrectionName = "Negative Lab Pro"
  L14_2.CorrectionReferenceX = 0.5
  L14_2.CorrectionReferenceY = 0.5
  L14_2.CorrectionSyncID = "4926101D005E4373AFCD6E51C47C18FE"
  L14_2.GreenCurve = L10_2
  L14_2.RedCurve = L9_2
  L14_2.LocalBlacks2012 = 0
  L14_2.LocalBrightness = 0
  L14_2.LocalClarity = 0
  L14_2.LocalClarity2012 = 0
  L14_2.LocalContrast = 0
  L14_2.LocalContrast2012 = 0
  L14_2.LocalCurveRefineSaturation = 100
  L14_2.LocalDefringe = 0
  L14_2.LocalDehaze = 0
  L14_2.LocalExposure = 0
  L14_2.LocalExposure2012 = 0
  L14_2.LocalGrain = 0
  L14_2.LocalHighlights2012 = 0
  L14_2.LocalHue = 0
  L14_2.LocalLuminanceNoise = 0
  L14_2.LocalMoire = 0
  L14_2.LocalSaturation = 0
  L14_2.LocalShadows2012 = 0
  L14_2.LocalSharpness = 0
  L14_2.LocalTemperature = 0
  L14_2.LocalTexture = 0
  L14_2.LocalTint = 0
  L14_2.LocalToningHue = 0
  L14_2.LocalToningSaturation = 0
  L14_2.LocalWhites2012 = 0
  L14_2.What = "Correction"
  if L13_2 ~= nil then
    L15_2 = type
    L16_2 = L13_2
    L15_2 = L15_2(L16_2)
    if L15_2 ~= "array" then
      goto lbl_221
    end
    L15_2 = #L13_2
    if L15_2 ~= 0 then
      goto lbl_221
    end
  end
  L15_2 = L7_1
  L16_2 = L15_2
  L15_2 = L15_2.trace
  L17_2 = "No previous mask"
  L15_2(L16_2, L17_2)
  L15_2 = {}
  L16_2 = L14_2
  L15_2[1] = L16_2
  L13_2 = L15_2
  L15_2 = _G
  L15_2 = L15_2.activePhotosMeta
  L15_2 = L15_2[A4_2]
  L15_2.existingNLPMaskIndex = 1
  goto lbl_241
  ::lbl_221::
  L15_2 = _G
  L15_2 = L15_2.activePhotosMeta
  L15_2 = L15_2[A4_2]
  L15_2 = L15_2.existingNLPMaskIndex
  if L15_2 ~= nil then
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[A4_2]
    L15_2 = L15_2.existingNLPMaskIndex
    L13_2[L15_2] = L14_2
  else
    L15_2 = #L13_2
    L15_2 = L15_2 + 1
    L13_2[L15_2] = L14_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[A4_2]
    L16_2 = #L13_2
    L15_2.existingNLPMaskIndex = L16_2
  end
  ::lbl_241::
  L5_2.MaskGroupBasedCorrections = L13_2
  L15_2 = _G
  L15_2 = L15_2.activePhotosMeta
  L15_2 = L15_2[A4_2]
  L15_2.activeMasks = L13_2
  L15_2 = L7_1
  L16_2 = L15_2
  L15_2 = L15_2.trace
  L17_2 = "Layermask Curves: "
  L18_2 = L2_1
  L19_2 = L7_2
  L18_2 = L18_2(L19_2)
  L17_2 = L17_2 .. L18_2
  L15_2(L16_2, L17_2)
  L15_2 = L7_1
  L16_2 = L15_2
  L15_2 = L15_2.trace
  L17_2 = "lightroomSettings: "
  L18_2 = L2_1
  L19_2 = L5_2
  L18_2 = L18_2(L19_2)
  L17_2 = L17_2 .. L18_2
  L15_2(L16_2, L17_2)
  return L5_2
end

L1_1.makeMaskCurve = L35_1

function L35_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A1_2]
  L2_2 = L2_2.activeMasks
  L3_2 = _G
  L3_2 = L3_2.activePhotosMeta
  L3_2 = L3_2[A1_2]
  L3_2 = L3_2.existingNLPMaskIndex
  L4_2 = type
  L5_2 = L2_2
  L4_2 = L4_2(L5_2)
  if L4_2 == "table" then
    L4_2 = #L2_2
    if L4_2 == 1 and L3_2 == 1 then
      L4_2 = {}
      L2_2 = L4_2
  end
  else
    L4_2 = type
    L5_2 = L2_2
    L4_2 = L4_2(L5_2)
    if L4_2 == "table" then
      L4_2 = #L2_2
      if 1 < L4_2 then
        L4_2 = table
        L4_2 = L4_2.remove
        L5_2 = L2_2
        L6_2 = L3_2
        L4_2(L5_2, L6_2)
    end
    else
      return A0_2
    end
  end
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A1_2]
  L4_2.existingNLPMaskIndex = nil
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A1_2]
  L4_2.activeMasks = L2_2
  A0_2.MaskGroupBasedCorrections = L2_2
  return A0_2
end

L1_1.removeMaskCurve = L35_1

function L35_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2)
  local L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2, L41_2, L42_2, L43_2
  L6_2 = L7_1
  L7_2 = L6_2
  L6_2 = L6_2.trace
  L8_2 = "in Update Tone curve"
  L6_2(L7_2, L8_2)
  L6_2 = L7_1
  L7_2 = L6_2
  L6_2 = L6_2.trace
  L8_2 = "ImageType: "
  L9_2 = A1_2.imageType
  L8_2 = L8_2 .. L9_2
  L6_2(L7_2, L8_2)
  if not A5_2 then
    L6_2 = _G
    A5_2 = L6_2.activePhotoIndex
  end
  L6_2 = L10_1
  L7_2 = A0_2
  L8_2 = A1_2
  L9_2 = A2_2
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  L7_2 = L7_1
  L8_2 = L7_2
  L7_2 = L7_2.trace
  L9_2 = "Break 1"
  L7_2(L8_2, L9_2)
  L7_2 = nil
  L8_2 = nil
  L9_2 = nil
  L10_2 = nil
  L11_2 = nil
  L12_2 = nil
  L13_2 = nil
  L14_2 = nil
  L15_2 = nil
  L16_2 = nil
  L17_2 = nil
  L18_2 = nil
  L19_2 = nil
  L20_2 = nil
  L21_2 = nil
  L22_2 = A1_2.clipMethod
  if L22_2 ~= nil then
    L22_2 = A1_2.clipMethod
    if L22_2 == "colorProtectedClipping" then
      L22_2 = L32_1
      L23_2 = A0_2
      L24_2 = A1_2
      L25_2 = A3_2
      L26_2 = A4_2
      L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2 = L22_2(L23_2, L24_2, L25_2, L26_2)
      L21_2 = L36_2
      L20_2 = L35_2
      L19_2 = L34_2
      L18_2 = L33_2
      L17_2 = L32_2
      L16_2 = L31_2
      L15_2 = L30_2
      L14_2 = L29_2
      L13_2 = L28_2
      L12_2 = L27_2
      L11_2 = L26_2
      L10_2 = L25_2
      L9_2 = L24_2
      L8_2 = L23_2
      L7_2 = L22_2
  end
  else
    L22_2 = L33_1
    L23_2 = A0_2
    L24_2 = A1_2
    L25_2 = A3_2
    L26_2 = A4_2
    L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2 = L22_2(L23_2, L24_2, L25_2, L26_2)
    L21_2 = L36_2
    L20_2 = L35_2
    L19_2 = L34_2
    L18_2 = L33_2
    L17_2 = L32_2
    L16_2 = L31_2
    L15_2 = L30_2
    L14_2 = L29_2
    L13_2 = L28_2
    L12_2 = L27_2
    L11_2 = L26_2
    L10_2 = L25_2
    L9_2 = L24_2
    L8_2 = L23_2
    L7_2 = L22_2
  end
  L22_2 = L11_1
  L23_2 = math
  L23_2 = L23_2.min
  L24_2 = L13_2
  L25_2 = L14_2
  L26_2 = L15_2
  L23_2 = L23_2(L24_2, L25_2, L26_2)
  L24_2 = A1_2
  L22_2 = L22_2(L23_2, L24_2)
  L23_2 = L17_1
  L24_2 = L22_2
  L23_2 = L23_2(L24_2)
  L24_2 = L7_1
  L25_2 = L24_2
  L24_2 = L24_2.trace
  L26_2 = "makeNodeArray: "
  L27_2 = L2_1
  L28_2 = L23_2
  L27_2 = L27_2(L28_2)
  L26_2 = L26_2 .. L27_2
  L24_2(L25_2, L26_2)
  L24_2 = A1_2.imageType
  if L24_2 == "positive" then
  end
  L24_2 = L34_1
  L25_2 = A0_2
  L26_2 = A1_2
  L27_2 = L23_2
  L28_2 = A3_2
  L29_2 = A4_2
  L24_2 = L24_2(L25_2, L26_2, L27_2, L28_2, L29_2)
  L25_2 = L7_1
  L26_2 = L25_2
  L25_2 = L25_2.trace
  L27_2 = "getRGBNodesForXAxis: "
  L28_2 = L2_1
  L29_2 = L24_2
  L28_2 = L28_2(L29_2)
  L27_2 = L27_2 .. L28_2
  L25_2(L26_2, L27_2)
  L25_2 = A1_2.toneProfile
  L26_2 = L5_1
  L26_2 = L26_2.toneProfiles
  L26_2 = L26_2[L25_2]
  L27_2 = {}
  L28_2 = L6_2.cyan
  L29_2 = L6_2.tint
  L30_2 = L6_2.temp
  L27_2[1] = L28_2
  L27_2[2] = L29_2
  L27_2[3] = L30_2
  L28_2 = A1_2.layerOrder
  if L28_2 == "colorFirst" then
    L28_2 = A1_2.wbTint
    if L28_2 == 0 then
      L28_2 = A1_2.wbTemp
      if L28_2 == 0 then
        goto lbl_136
      end
    end
    L28_2 = L1_1
    L28_2 = L28_2.colorLinearLayer
    L29_2 = L23_2
    L30_2 = A1_2
    L28_2 = L28_2(L29_2, L30_2)
    L23_2 = L28_2
    ::lbl_136::
    L28_2 = L1_1
    L28_2 = L28_2.colorGammaLayer
    L29_2 = L23_2
    L30_2 = L27_2
    L28_2 = L28_2(L29_2, L30_2)
    L23_2 = L28_2
  end
  L28_2 = L1_1
  L28_2 = L28_2.exposureLayer
  L29_2 = L23_2
  L30_2 = L6_2.exposure
  L31_2 = A1_2
  L28_2 = L28_2(L29_2, L30_2, L31_2)
  L23_2 = L28_2
  L28_2 = L1_1
  L28_2 = L28_2.gammaLayer
  L29_2 = L23_2
  L30_2 = L6_2.brightness
  L31_2 = A1_2
  L28_2 = L28_2(L29_2, L30_2, L31_2)
  L23_2 = L28_2
  L28_2 = L1_1
  L28_2 = L28_2.contrastLayer
  L29_2 = L23_2
  L30_2 = L6_2.contrast
  L28_2 = L28_2(L29_2, L30_2)
  L23_2 = L28_2
  L28_2 = L7_1
  L29_2 = L28_2
  L28_2 = L28_2.trace
  L30_2 = "Contrast Layer: "
  L31_2 = L2_1
  L32_2 = L23_2
  L31_2 = L31_2(L32_2)
  L30_2 = L30_2 .. L31_2
  L28_2(L29_2, L30_2)
  L28_2 = L20_1
  L29_2 = L23_2
  L30_2 = L6_2.highlights
  L28_2 = L28_2(L29_2, L30_2)
  L23_2 = L28_2
  L28_2 = L21_1
  L29_2 = L23_2
  L30_2 = L6_2.shadows
  L28_2 = L28_2(L29_2, L30_2)
  L23_2 = L28_2
  L28_2 = L7_1
  L29_2 = L28_2
  L28_2 = L28_2.trace
  L30_2 = "Highlights and Shadows Layer: "
  L31_2 = L2_1
  L32_2 = L23_2
  L31_2 = L31_2(L32_2)
  L30_2 = L30_2 .. L31_2
  L28_2(L29_2, L30_2)
  L28_2 = L22_1
  L29_2 = L23_2
  L30_2 = L6_2
  L28_2 = L28_2(L29_2, L30_2)
  L23_2 = L28_2
  L28_2 = L28_1
  L29_2 = L23_2
  L30_2 = A1_2
  L31_2 = L6_2
  L32_2 = L26_2
  L28_2 = L28_2(L29_2, L30_2, L31_2, L32_2)
  L23_2 = L28_2
  L28_2 = L7_1
  L29_2 = L28_2
  L28_2 = L28_2.trace
  L30_2 = "Black and White Layer: "
  L31_2 = L2_1
  L32_2 = L23_2
  L31_2 = L31_2(L32_2)
  L30_2 = L30_2 .. L31_2
  L28_2(L29_2, L30_2)
  L28_2 = nil
  L29_2 = nil
  L30_2 = A1_2.imageType
  if L30_2 ~= "positive" then
    L30_2 = A1_2.imageType
    if L30_2 ~= "editOnly" then
      goto lbl_228
    end
  end
  L30_2 = {}
  L31_2 = L19_2
  L32_2 = L20_2
  L33_2 = L21_2
  L30_2[1] = L31_2
  L30_2[2] = L32_2
  L30_2[3] = L33_2
  L28_2 = L30_2
  L30_2 = {}
  L31_2 = L16_2
  L32_2 = L17_2
  L33_2 = L18_2
  L30_2[1] = L31_2
  L30_2[2] = L32_2
  L30_2[3] = L33_2
  L29_2 = L30_2
  goto lbl_240
  ::lbl_228::
  L30_2 = {}
  L31_2 = L16_2
  L32_2 = L17_2
  L33_2 = L18_2
  L30_2[1] = L31_2
  L30_2[2] = L32_2
  L30_2[3] = L33_2
  L28_2 = L30_2
  L30_2 = {}
  L31_2 = L19_2
  L32_2 = L20_2
  L33_2 = L21_2
  L30_2[1] = L31_2
  L30_2[2] = L32_2
  L30_2[3] = L33_2
  L29_2 = L30_2
  ::lbl_240::
  L30_2 = L29_1
  L31_2 = L23_2
  L32_2 = L28_2
  L33_2 = L29_2
  L30_2 = L30_2(L31_2, L32_2, L33_2)
  L23_2 = L30_2
  L30_2 = L7_1
  L31_2 = L30_2
  L30_2 = L30_2.trace
  L32_2 = "Linear Layer: "
  L33_2 = L2_1
  L34_2 = L23_2
  L33_2 = L33_2(L34_2)
  L32_2 = L32_2 .. L33_2
  L30_2(L31_2, L32_2)
  L30_2 = A1_2.layerOrder
  if L30_2 == "tonesFirst" then
    L30_2 = L7_1
    L31_2 = L30_2
    L30_2 = L30_2.trace
    L32_2 = "tonesFirst"
    L30_2(L31_2, L32_2)
    L30_2 = L17_1
    L31_2 = L22_2
    L30_2 = L30_2(L31_2)
    L31_2 = L7_1
    L32_2 = L31_2
    L31_2 = L31_2.trace
    L33_2 = "Back from makeNodeArray"
    L31_2(L32_2, L33_2)
    L31_2 = A1_2.wbTint
    if L31_2 == 0 then
      L31_2 = A1_2.wbTemp
      if L31_2 == 0 then
        goto lbl_289
      end
    end
    L31_2 = L7_1
    L32_2 = L31_2
    L31_2 = L31_2.trace
    L33_2 = "wbTint or wbTemp non-zero"
    L31_2(L32_2, L33_2)
    L31_2 = L1_1
    L31_2 = L31_2.colorLinearLayer
    L32_2 = L30_2
    L33_2 = A1_2
    L31_2 = L31_2(L32_2, L33_2)
    L32_2 = L19_1
    L33_2 = L23_2
    L34_2 = L31_2
    L35_2 = L30_2
    L32_2 = L32_2(L33_2, L34_2, L35_2)
    L23_2 = L32_2
    ::lbl_289::
    L31_2 = L1_1
    L31_2 = L31_2.colorGammaLayer
    L32_2 = L30_2
    L33_2 = L27_2
    L31_2 = L31_2(L32_2, L33_2)
    L32_2 = L7_1
    L33_2 = L32_2
    L32_2 = L32_2.trace
    L34_2 = "Back from colorGammaLayer"
    L32_2(L33_2, L34_2)
    L32_2 = L19_1
    L33_2 = L23_2
    L34_2 = L31_2
    L35_2 = L30_2
    L32_2 = L32_2(L33_2, L34_2, L35_2)
    L23_2 = L32_2
    L32_2 = L7_1
    L33_2 = L32_2
    L32_2 = L32_2.trace
    L34_2 = "Back from addNodeArraysWithBase"
    L32_2(L33_2, L34_2)
  end
  L30_2 = L7_1
  L31_2 = L30_2
  L30_2 = L30_2.trace
  L32_2 = "Tones First Layer: "
  L33_2 = L2_1
  L34_2 = L23_2
  L33_2 = L33_2(L34_2)
  L32_2 = L32_2 .. L33_2
  L30_2(L31_2, L32_2)
  L30_2 = A1_2.highlightCyan
  if L30_2 == 0 then
    L30_2 = A1_2.highlightTint
    if L30_2 == 0 then
      L30_2 = A1_2.highlightTemp
      if L30_2 == 0 then
        goto lbl_331
      end
    end
  end
  L30_2 = L27_1
  L31_2 = L23_2
  L32_2 = A1_2
  L33_2 = L26_2
  L30_2 = L30_2(L31_2, L32_2, L33_2)
  L23_2 = L30_2
  ::lbl_331::
  L30_2 = A1_2.shadowCyan
  if L30_2 == 0 then
    L30_2 = A1_2.shadowTint
    if L30_2 == 0 then
      L30_2 = A1_2.shadowTemp
      if L30_2 == 0 then
        goto lbl_346
      end
    end
  end
  L30_2 = L26_1
  L31_2 = L23_2
  L32_2 = A1_2
  L33_2 = L26_2
  L30_2 = L30_2(L31_2, L32_2, L33_2)
  L23_2 = L30_2
  ::lbl_346::
  L30_2 = L7_1
  L31_2 = L30_2
  L30_2 = L30_2.trace
  L32_2 = "Black and White Toning Layer: "
  L33_2 = L2_1
  L34_2 = L23_2
  L33_2 = L33_2(L34_2)
  L32_2 = L32_2 .. L33_2
  L30_2(L31_2, L32_2)
  L30_2 = A1_2.correctionType
  if L30_2 == "multiPass" then
    L30_2 = IsGroupMaskAvailable
    if L30_2 == true then
      L30_2 = A1_2.imageType
      if L30_2 ~= "positive" then
        L30_2 = A1_2.imageType
        if L30_2 ~= "editOnly" then
          L30_2 = L1_1
          L30_2 = L30_2.makeMaskCurve
          L31_2 = L23_2
          L32_2 = L24_2
          L33_2 = A0_2
          L34_2 = A1_2
          L35_2 = A5_2
          L30_2 = L30_2(L31_2, L32_2, L33_2, L34_2, L35_2)
          return L30_2
      end
    end
  end
  else
    L30_2 = L7_1
    L31_2 = L30_2
    L30_2 = L30_2.trace
    L32_2 = "NO MASK CURVE"
    L30_2(L31_2, L32_2)
    L30_2 = A1_2.imageType
    if L30_2 ~= "positive" then
      L30_2 = A1_2.imageType
      if L30_2 ~= "editOnly" then
        goto lbl_391
      end
    end
    L30_2 = L7_1
    L31_2 = L30_2
    L30_2 = L30_2.trace
    L32_2 = "DO NOT INVERT NODES"
    L30_2(L31_2, L32_2)
    goto lbl_400
    ::lbl_391::
    L30_2 = L1_1
    L30_2 = L30_2.invert
    L31_2 = L23_2
    L30_2 = L30_2(L31_2)
    L23_2 = L30_2
    L30_2 = L7_1
    L31_2 = L30_2
    L30_2 = L30_2.trace
    L32_2 = "INVERT NODES"
    L30_2(L31_2, L32_2)
    ::lbl_400::
    L30_2 = L7_1
    L31_2 = L30_2
    L30_2 = L30_2.trace
    L32_2 = "Inverted Nodes "
    L33_2 = L2_1
    L34_2 = L23_2
    L33_2 = L33_2(L34_2)
    L32_2 = L32_2 .. L33_2
    L30_2(L31_2, L32_2)
    L30_2 = {}
    L31_2 = A0_2[1]
    L31_2 = L31_2.settingName
    L32_2 = {}
    L30_2[L31_2] = L32_2
    L31_2 = A0_2[2]
    L31_2 = L31_2.settingName
    L32_2 = {}
    L30_2[L31_2] = L32_2
    L31_2 = A0_2[3]
    L31_2 = L31_2.settingName
    L32_2 = {}
    L30_2[L31_2] = L32_2
    L31_2 = 1
    L32_2 = #A0_2
    L33_2 = 1
    for L34_2 = L31_2, L32_2, L33_2 do
      L35_2 = 1
      L36_2 = L23_2[1]
      L36_2 = #L36_2
      L37_2 = 1
      for L38_2 = L35_2, L36_2, L37_2 do
        L39_2 = L7_1
        L40_2 = L39_2
        L39_2 = L39_2.trace
        L41_2 = "ROUND: "
        L42_2 = L24_2[L34_2]
        L42_2 = L42_2[L38_2]
        L41_2 = L41_2 .. L42_2
        L39_2(L40_2, L41_2)
        L39_2 = table
        L39_2 = L39_2.insert
        L40_2 = A0_2[L34_2]
        L40_2 = L40_2.settingName
        L40_2 = L30_2[L40_2]
        L41_2 = L3_1
        L41_2 = L41_2.round
        L42_2 = L24_2[L34_2]
        L42_2 = L42_2[L38_2]
        L41_2, L42_2, L43_2 = L41_2(L42_2)
        L39_2(L40_2, L41_2, L42_2, L43_2)
        L39_2 = L23_2[L34_2]
        L39_2 = L39_2[L38_2]
        L39_2 = L39_2 * 255
        L40_2 = table
        L40_2 = L40_2.insert
        L41_2 = A0_2[L34_2]
        L41_2 = L41_2.settingName
        L41_2 = L30_2[L41_2]
        L42_2 = L3_1
        L42_2 = L42_2.round
        L43_2 = L39_2
        L42_2, L43_2 = L42_2(L43_2)
        L40_2(L41_2, L42_2, L43_2)
      end
    end
    L31_2 = _G
    L31_2 = L31_2.activePhotosMeta
    L31_2 = L31_2[A5_2]
    L31_2 = L31_2.existingNLPMaskIndex
    if L31_2 ~= nil then
      L31_2 = NLPViewController
      L31_2 = L31_2.removeMaskCurve
      L32_2 = L30_2
      L33_2 = A5_2
      L31_2 = L31_2(L32_2, L33_2)
      L30_2 = L31_2
      L31_2 = _G
      L31_2 = L31_2.activePhotosMeta
      L31_2 = L31_2[A5_2]
      L31_2.existingNLPMaskIndex = nil
    end
    L31_2 = L7_1
    L32_2 = L31_2
    L31_2 = L31_2.trace
    L33_2 = "LIGHTROOM SETTINGS: "
    L34_2 = L2_1
    L35_2 = L30_2
    L34_2 = L34_2(L35_2)
    L33_2 = L33_2 .. L34_2
    L31_2(L32_2, L33_2)
    return L30_2
  end
end

L1_1.updateToneCurve = L35_1
return L1_1
