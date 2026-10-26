local L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1

L1_1 = require
L2_1 = "UtilV3"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrColor"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrLogger"
L3_1 = L3_1(L4_1)
L4_1 = require
L5_1 = "inspect"
L4_1 = L4_1(L5_1)
L5_1 = L3_1
L6_1 = "Negative"
L5_1 = L5_1(L6_1)
L6_1 = _G
L6_1 = L6_1.dev
if L6_1 == true then
  L7_1 = L5_1
  L6_1 = L5_1.enable
  L8_1 = "logfile"
  L6_1(L7_1, L8_1)
else
  L7_1 = L5_1
  L6_1 = L5_1.disable
  L6_1(L7_1)
end
L6_1 = {}
L6_1.__index = L6_1

function L7_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = setmetatable
  L4_2 = {}
  L5_2 = A0_2 or L5_2
  if not A0_2 then
    L5_2 = 0
  end
  L4_2.r = L5_2
  L5_2 = A1_2 or L5_2
  if not A1_2 then
    L5_2 = 0
  end
  L4_2.g = L5_2
  L5_2 = A2_2 or L5_2
  if not A2_2 then
    L5_2 = 0
  end
  L4_2.b = L5_2
  L5_2 = L6_1
  return L3_2(L4_2, L5_2)
end

function L8_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2
  L3_2 = L7_1
  L4_2 = A0_2 / 255
  L5_2 = A1_2 / 255
  L6_2 = A2_2 / 255
  return L3_2(L4_2, L5_2, L6_2)
end

function L9_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = L7_1
  L3_2 = A0_2
  L2_2 = A0_2.red
  L2_2 = L2_2(L3_2)
  L4_2 = A0_2
  L3_2 = A0_2.green
  L3_2 = L3_2(L4_2)
  L5_2 = A0_2
  L4_2 = A0_2.blue
  L4_2, L5_2 = L4_2(L5_2)
  return L1_2(L2_2, L3_2, L4_2, L5_2)
end

function L10_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L2_2 = A0_2
  L1_2 = A0_2.gsub
  L3_2 = "#"
  L4_2 = ""
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  L3_2 = L1_2
  L2_2 = L1_2.len
  L2_2 = L2_2(L3_2)
  if L2_2 == 3 then
    L2_2 = tonumber
    L3_2 = "0x"
    L5_2 = L1_2
    L4_2 = L1_2.sub
    L6_2 = 1
    L7_2 = 1
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    L3_2 = L3_2 .. L4_2
    L2_2 = L2_2(L3_2)
    L2_2 = L2_2 * 17
    L2_2 = L2_2 / 255
    L3_2 = tonumber
    L4_2 = "0x"
    L6_2 = L1_2
    L5_2 = L1_2.sub
    L7_2 = 2
    L8_2 = 2
    L5_2 = L5_2(L6_2, L7_2, L8_2)
    L4_2 = L4_2 .. L5_2
    L3_2 = L3_2(L4_2)
    L3_2 = L3_2 * 17
    L3_2 = L3_2 / 255
    L4_2 = tonumber
    L5_2 = "0x"
    L7_2 = L1_2
    L6_2 = L1_2.sub
    L8_2 = 3
    L9_2 = 3
    L6_2 = L6_2(L7_2, L8_2, L9_2)
    L5_2 = L5_2 .. L6_2
    L4_2 = L4_2(L5_2)
    L4_2 = L4_2 * 17
    L4_2 = L4_2 / 255
  else
    L2_2 = tonumber
    L3_2 = "0x"
    L5_2 = L1_2
    L4_2 = L1_2.sub
    L6_2 = 1
    L7_2 = 2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    L3_2 = L3_2 .. L4_2
    L2_2 = L2_2(L3_2)
    L2_2 = L2_2 / 255
    L3_2 = tonumber
    L4_2 = "0x"
    L6_2 = L1_2
    L5_2 = L1_2.sub
    L7_2 = 3
    L8_2 = 4
    L5_2 = L5_2(L6_2, L7_2, L8_2)
    L4_2 = L4_2 .. L5_2
    L3_2 = L3_2(L4_2)
    L3_2 = L3_2 / 255
    L4_2 = tonumber
    L5_2 = "0x"
    L7_2 = L1_2
    L6_2 = L1_2.sub
    L8_2 = 5
    L9_2 = 6
    L6_2 = L6_2(L7_2, L8_2, L9_2)
    L5_2 = L5_2 .. L6_2
    L4_2 = L4_2(L5_2)
    L4_2 = L4_2 / 255
  end
  L2_2 = L7_1
  L3_2 = r
  L4_2 = g
  L5_2 = b
  return L2_2(L3_2, L4_2, L5_2)
end

function L11_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  if A1_2 == 0 then
    L3_2 = L7_1
    L4_2 = A2_2
    L5_2 = A2_2
    L6_2 = A2_2
    return L3_2(L4_2, L5_2, L6_2)
  end
  if A0_2 == 360 then
    A0_2 = 0
  end
  L3_2 = math
  L3_2 = L3_2.floor
  L4_2 = A0_2 / 60
  L3_2 = L3_2(L4_2)
  L4_2 = A0_2 / 60
  L4_2 = L4_2 - L3_2
  L5_2 = 1 - A1_2
  L5_2 = A2_2 * L5_2
  L6_2 = A1_2 * L4_2
  L6_2 = 1 - L6_2
  L6_2 = A2_2 * L6_2
  L7_2 = 1 - L4_2
  L7_2 = A1_2 * L7_2
  L7_2 = 1 - L7_2
  L7_2 = A2_2 * L7_2
  if L3_2 == 0 then
    L8_2 = L7_1
    L9_2 = A2_2
    L10_2 = L7_2
    L11_2 = L5_2
    return L8_2(L9_2, L10_2, L11_2)
  elseif L3_2 == 1 then
    L8_2 = L7_1
    L9_2 = L6_2
    L10_2 = A2_2
    L11_2 = L5_2
    return L8_2(L9_2, L10_2, L11_2)
  elseif L3_2 == 2 then
    L8_2 = L7_1
    L9_2 = L5_2
    L10_2 = A2_2
    L11_2 = L7_2
    return L8_2(L9_2, L10_2, L11_2)
  elseif L3_2 == 3 then
    L8_2 = L7_1
    L9_2 = L5_2
    L10_2 = L6_2
    L11_2 = A2_2
    return L8_2(L9_2, L10_2, L11_2)
  elseif L3_2 == 4 then
    L8_2 = L7_1
    L9_2 = L7_2
    L10_2 = L5_2
    L11_2 = A2_2
    return L8_2(L9_2, L10_2, L11_2)
  elseif L3_2 == 5 then
    L8_2 = L7_1
    L9_2 = A2_2
    L10_2 = L5_2
    L11_2 = L6_2
    return L8_2(L9_2, L10_2, L11_2)
  end
end

function L12_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = "("
  L2_2 = tonumber
  L3_2 = A0_2.r
  L2_2 = L2_2(L3_2)
  L3_2 = ","
  L4_2 = tonumber
  L5_2 = A0_2.g
  L4_2 = L4_2(L5_2)
  L5_2 = ","
  L6_2 = tonumber
  L7_2 = A0_2.b
  L6_2 = L6_2(L7_2)
  L7_2 = ")"
  L1_2 = L1_2 .. L2_2 .. L3_2 .. L4_2 .. L5_2 .. L6_2 .. L7_2
  return L1_2
end

L6_1.__tostring = L12_1

function L12_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = L2_1
  L2_2 = A0_2.r
  L3_2 = A0_2.g
  L4_2 = A0_2.b
  return L1_2(L2_2, L3_2, L4_2)
end

L6_1.toLrColor = L12_1

function L12_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L1_2 = A0_2.r
  L2_2 = A0_2.g
  L3_2 = A0_2.b
  L4_2 = nil
  L5_2 = nil
  L6_2 = nil
  L7_2 = math
  L7_2 = L7_2.min
  L8_2 = L1_2
  L9_2 = L2_2
  L10_2 = L3_2
  L7_2 = L7_2(L8_2, L9_2, L10_2)
  L8_2 = math
  L8_2 = L8_2.max
  L9_2 = L1_2
  L10_2 = L2_2
  L11_2 = L3_2
  L8_2 = L8_2(L9_2, L10_2, L11_2)
  L6_2 = L8_2
  L9_2 = L8_2 - L7_2
  if L8_2 ~= 0 then
    L5_2 = L9_2 / L8_2
  else
    L5_2 = 0
    L4_2 = -1
    L10_2 = L4_2
    L11_2 = L5_2
    L12_2 = L6_2
    return L10_2, L11_2, L12_2
  end
  if L1_2 == L8_2 then
    L10_2 = L2_2 - L3_2
    L4_2 = L10_2 / L9_2
  elseif L2_2 == L8_2 then
    L10_2 = L3_2 - L1_2
    L10_2 = L10_2 / L9_2
    L4_2 = 2 + L10_2
  else
    L10_2 = L1_2 - L2_2
    L10_2 = L10_2 / L9_2
    L4_2 = 4 + L10_2
  end
  L4_2 = L4_2 * 60
  if L4_2 < 0 then
    L4_2 = L4_2 + 360
  end
  L10_2 = L4_2
  L11_2 = L5_2
  L12_2 = L6_2
  return L10_2, L11_2, L12_2
end

L6_1.toHSV = L12_1

function L12_1(A0_2)
  local L1_2, L2_2
  L1_2 = A0_2.r
  L2_2 = A0_2.g
  L1_2 = L1_2 + L2_2
  L2_2 = A0_2.b
  L1_2 = L1_2 + L2_2
  L1_2 = L1_2 / 3
  return L1_2
end

L6_1.avg = L12_1

function L12_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = A0_2.r
  L2_2 = A0_2.g
  L3_2 = A0_2.b
  return L1_2, L2_2, L3_2
end

L6_1.unpack = L12_1

function L12_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = {}
  L3_2 = nil
  if A1_2 == nil then
    L5_2 = A0_2
    L4_2 = A0_2.avg
    L4_2 = L4_2(L5_2)
    L3_2 = L4_2 * 255
  else
    L4_2 = A0_2[A1_2]
    L3_2 = L4_2 * 255
  end
  L4_2 = A0_2.r
  if L4_2 == 0 then
    L4_2 = A0_2.g
    if L4_2 == 0 then
      L4_2 = A0_2.b
      if L4_2 == 0 then
        L4_2 = {}
        L5_2 = 0
        L6_2 = 0
        L7_2 = 0
        L4_2[1] = L5_2
        L4_2[2] = L6_2
        L4_2[3] = L7_2
        return L4_2
      end
    end
  end
  L4_2 = {}
  L5_2 = A0_2.r
  L5_2 = L5_2 * 255
  L5_2 = L3_2 - L5_2
  L6_2 = A0_2.g
  L6_2 = L6_2 * 255
  L6_2 = L3_2 - L6_2
  L7_2 = A0_2.b
  L7_2 = L7_2 * 255
  L7_2 = L3_2 - L7_2
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L2_2 = L4_2
  return L2_2
end

L6_1.linearBalance = L12_1

function L12_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = {}
  L3_2 = {}
  L4_2 = nil
  if A1_2 == nil then
    L6_2 = A0_2
    L5_2 = A0_2.avg
    L5_2 = L5_2(L6_2)
    L4_2 = L5_2
  else
    L4_2 = A0_2[A1_2]
  end
  L5_2 = A0_2.r
  if L5_2 == 0 then
    L5_2 = A0_2.g
    if L5_2 == 0 then
      L5_2 = A0_2.b
      if L5_2 == 0 then
        L5_2 = {}
        L6_2 = 1
        L7_2 = 1
        L8_2 = 1
        L5_2[1] = L6_2
        L5_2[2] = L7_2
        L5_2[3] = L8_2
        return L5_2
      end
    end
  end
  L5_2 = {}
  L6_2 = L1_1
  L6_2 = L6_2.logb
  L7_2 = L4_2
  L8_2 = A0_2.r
  L6_2 = L6_2(L7_2, L8_2)
  L6_2 = 1 / L6_2
  L7_2 = L1_1
  L7_2 = L7_2.logb
  L8_2 = L4_2
  L9_2 = A0_2.g
  L7_2 = L7_2(L8_2, L9_2)
  L7_2 = 1 / L7_2
  L8_2 = L1_1
  L8_2 = L8_2.logb
  L9_2 = L4_2
  L10_2 = A0_2.b
  L8_2 = L8_2(L9_2, L10_2)
  L8_2 = 1 / L8_2
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L2_2 = L5_2
  return L2_2
end

L6_1.gammaBalance = L12_1
L12_1 = {}
L13_1 = {}
L14_1 = {}
L15_1 = 0.4360747
L16_1 = 0.3850649
L17_1 = 0.1430804
L18_1 = 0.2225045
L19_1 = 0.7168786
L20_1 = 0.0606169
L21_1 = 0.0139322
L22_1 = 0.0971045
L23_1 = 0.7141733
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L15_1 = 3.1338561
L16_1 = -1.6168667
L17_1 = -0.4906146
L18_1 = -0.9787684
L19_1 = 1.9161415
L20_1 = 0.033454
L21_1 = 0.0719453
L22_1 = -0.2289914
L23_1 = 1.4052427
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.04045 then
    L1_2 = A0_2 / 12.92
    return L1_2
  else
    L1_2 = A0_2 + 0.05213270142180095
    L1_2 = L1_2 ^ 2.4
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.0031308 then
    L1_2 = 12.92 * A0_2
    return L1_2
  else
    L1_2 = A0_2 ^ 0.4166666666666667
    L1_2 = 1.055 * L1_2
    L1_2 = L1_2 - 0.055
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.sRGBd50 = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.7976749
L16_1 = 0.1351917
L17_1 = 0.0313534
L18_1 = 0.2880402
L19_1 = 0.7118741
L20_1 = 8.57E-5
L21_1 = 0
L22_1 = 0
L23_1 = 0.82521
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L15_1 = 1.3459433
L16_1 = -0.2556075
L17_1 = -0.0511118
L18_1 = -0.5445989
L19_1 = 1.5081673
L20_1 = 0.0205351
L21_1 = 0
L22_1 = 0
L23_1 = 1.2118128
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 1.8
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 0.5555555555555556
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.ProPhoto = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.7980403
L16_1 = 0.1349972
L17_1 = 0.0309624
L18_1 = 0.2880146
L19_1 = 0.7119854
L20_1 = 0
L21_1 = 0
L22_1 = 0
L23_1 = 0.824
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L15_1 = 1.345115
L16_1 = -0.2550429
L17_1 = -0.0505437
L18_1 = -0.5441301
L19_1 = 1.5076938
L20_1 = 0.0204461
L21_1 = 0
L22_1 = 0
L23_1 = 1.2135922
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 1.8
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 0.5555555555555556
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.ProPhotoCalc = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.5151
L16_1 = 0.292
L17_1 = 0.1571
L18_1 = 0.2412
L19_1 = 0.6922
L20_1 = 0.0666
L21_1 = -0.0011
L22_1 = 0.0419
L23_1 = 0.7841
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.04045 then
    L1_2 = A0_2 / 12.92
    return L1_2
  else
    L1_2 = A0_2 + 0.05213270142180095
    L1_2 = L1_2 ^ 2.4
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.0031308 then
    L1_2 = 12.92 * A0_2
    return L1_2
  else
    L1_2 = A0_2 ^ 0.4166666666666667
    L1_2 = 1.055 * L1_2
    L1_2 = L1_2 - 0.055
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.displayP3 = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.7976749
L16_1 = 0.1351917
L17_1 = 0.0313534
L18_1 = 0.2880402
L19_1 = 0.7118741
L20_1 = 8.57E-5
L21_1 = 0
L22_1 = 0
L23_1 = 0.82521
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L15_1 = 1.3459433
L16_1 = -0.2556075
L17_1 = -0.0511118
L18_1 = -0.5445989
L19_1 = 1.5081673
L20_1 = 0.0205351
L21_1 = 0
L22_1 = 0
L23_1 = 1.2118128
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.04045 then
    L1_2 = A0_2 / 12.92
    return L1_2
  else
    L1_2 = A0_2 + 0.05213270142180095
    L1_2 = L1_2 ^ 2.4
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0.0031308 then
    L1_2 = 12.92 * A0_2
    return L1_2
  else
    L1_2 = A0_2 ^ 0.4166666666666667
    L1_2 = 1.055 * L1_2
    L1_2 = L1_2 - 0.055
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.MelissaRGB = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.5151
L16_1 = 0.292
L17_1 = 0.1571
L18_1 = 0.2412
L19_1 = 0.6922
L20_1 = 0.0666
L21_1 = -0.0011
L22_1 = 0.0419
L23_1 = 0.7841
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = 2.4
  L2_2 = 0.948
  L3_2 = 0.052
  L4_2 = 0.077
  L5_2 = 0.04
  if A0_2 < L5_2 then
    L6_2 = L4_2 * A0_2
    return L6_2
  else
    L6_2 = L2_2 * A0_2
    L6_2 = L6_2 + L3_2
    L6_2 = L6_2 ^ L1_2
    return L6_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
end

L13_1.toGamma = L14_1
L12_1.displayP3gamma = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.3770584
L16_1 = 0.3583388
L17_1 = 0.2146028
L18_1 = 0.2009871
L19_1 = 0.6841936
L20_1 = 0.1148193
L21_1 = 0.0124579
L22_1 = 0.0923763
L23_1 = 0.9841658
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 1.801
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 0.5552470849528041
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.genericRGB = L13_1
L13_1 = {}
L14_1 = {}
L15_1 = 0.509
L16_1 = 0.321
L17_1 = 0.134
L18_1 = 0.275
L19_1 = 0.658
L20_1 = 0.067
L21_1 = 0.024
L22_1 = 0.109
L23_1 = 0.692
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.toXYZ = L14_1
L14_1 = {}
L15_1 = 2.6451479
L16_1 = -1.2252155
L17_1 = -0.3935844
L18_1 = -1.1140212
L19_1 = 2.0605365
L20_1 = 0.016218
L21_1 = 0.0837352
L22_1 = -0.2820713
L23_1 = 1.4561825
L14_1[1] = L15_1
L14_1[2] = L16_1
L14_1[3] = L17_1
L14_1[4] = L18_1
L14_1[5] = L19_1
L14_1[6] = L20_1
L14_1[7] = L21_1
L14_1[8] = L22_1
L14_1[9] = L23_1
L13_1.fromXYZ = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 1.8
    return L1_2
  end
end

L13_1.toLinear = L14_1

function L14_1(A0_2)
  local L1_2
  if A0_2 <= 0 then
    L1_2 = 0
    return L1_2
  else
    L1_2 = A0_2 ^ 0.5555555555555556
    return L1_2
  end
end

L13_1.toGamma = L14_1
L12_1.ColorMatch = L13_1

function L13_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2
  L3_2 = L12_1
  L3_2 = L3_2[A1_2]
  input = L3_2
  L3_2 = L12_1
  L3_2 = L3_2[A2_2]
  output = L3_2
  L3_2 = input
  L3_2 = L3_2.toLinear
  L4_2 = A0_2.r
  L3_2 = L3_2(L4_2)
  L4_2 = input
  L4_2 = L4_2.toLinear
  L5_2 = A0_2.g
  L4_2 = L4_2(L5_2)
  L5_2 = input
  L5_2 = L5_2.toLinear
  L6_2 = A0_2.b
  L5_2 = L5_2(L6_2)
  L6_2 = input
  L6_2 = L6_2.toXYZ
  L6_2 = L6_2[1]
  L6_2 = L3_2 * L6_2
  L7_2 = input
  L7_2 = L7_2.toXYZ
  L7_2 = L7_2[2]
  L7_2 = L4_2 * L7_2
  L6_2 = L6_2 + L7_2
  L7_2 = input
  L7_2 = L7_2.toXYZ
  L7_2 = L7_2[3]
  L7_2 = L5_2 * L7_2
  L6_2 = L6_2 + L7_2
  L7_2 = input
  L7_2 = L7_2.toXYZ
  L7_2 = L7_2[4]
  L7_2 = L3_2 * L7_2
  L8_2 = input
  L8_2 = L8_2.toXYZ
  L8_2 = L8_2[5]
  L8_2 = L4_2 * L8_2
  L7_2 = L7_2 + L8_2
  L8_2 = input
  L8_2 = L8_2.toXYZ
  L8_2 = L8_2[6]
  L8_2 = L5_2 * L8_2
  L7_2 = L7_2 + L8_2
  L8_2 = input
  L8_2 = L8_2.toXYZ
  L8_2 = L8_2[7]
  L8_2 = L3_2 * L8_2
  L9_2 = input
  L9_2 = L9_2.toXYZ
  L9_2 = L9_2[8]
  L9_2 = L4_2 * L9_2
  L8_2 = L8_2 + L9_2
  L9_2 = input
  L9_2 = L9_2.toXYZ
  L9_2 = L9_2[9]
  L9_2 = L5_2 * L9_2
  L8_2 = L8_2 + L9_2
  L9_2 = output
  L9_2 = L9_2.fromXYZ
  L9_2 = L9_2[1]
  L9_2 = L6_2 * L9_2
  L10_2 = output
  L10_2 = L10_2.fromXYZ
  L10_2 = L10_2[2]
  L10_2 = L7_2 * L10_2
  L9_2 = L9_2 + L10_2
  L10_2 = output
  L10_2 = L10_2.fromXYZ
  L10_2 = L10_2[3]
  L10_2 = L8_2 * L10_2
  L9_2 = L9_2 + L10_2
  L10_2 = output
  L10_2 = L10_2.fromXYZ
  L10_2 = L10_2[4]
  L10_2 = L6_2 * L10_2
  L11_2 = output
  L11_2 = L11_2.fromXYZ
  L11_2 = L11_2[5]
  L11_2 = L7_2 * L11_2
  L10_2 = L10_2 + L11_2
  L11_2 = output
  L11_2 = L11_2.fromXYZ
  L11_2 = L11_2[6]
  L11_2 = L8_2 * L11_2
  L10_2 = L10_2 + L11_2
  L11_2 = output
  L11_2 = L11_2.fromXYZ
  L11_2 = L11_2[7]
  L11_2 = L6_2 * L11_2
  L12_2 = output
  L12_2 = L12_2.fromXYZ
  L12_2 = L12_2[8]
  L12_2 = L7_2 * L12_2
  L11_2 = L11_2 + L12_2
  L12_2 = output
  L12_2 = L12_2.fromXYZ
  L12_2 = L12_2[9]
  L12_2 = L8_2 * L12_2
  L11_2 = L11_2 + L12_2
  L12_2 = output
  L12_2 = L12_2.toGamma
  L13_2 = L9_2
  L12_2 = L12_2(L13_2)
  L13_2 = output
  L13_2 = L13_2.toGamma
  L14_2 = L10_2
  L13_2 = L13_2(L14_2)
  L14_2 = output
  L14_2 = L14_2.toGamma
  L15_2 = L11_2
  L14_2 = L14_2(L15_2)
  L15_2 = L12_2
  L16_2 = L13_2
  L17_2 = L14_2
  return L15_2, L16_2, L17_2
end

L6_1.convertSpace = L13_1
L13_1 = setmetatable
L14_1 = {}
L14_1.new = L7_1
L14_1.from255 = L8_1
L14_1.fromHSV = L11_1
L14_1.fromHex = L10_1
L14_1.fromLrColor = L9_1
L15_1 = {}

function L16_1(A0_2, ...)
  local L2_2, L3_2
  L2_2 = L7_1
  L3_2 = ...
  return L2_2(L3_2)
end

L15_1.__call = L16_1
return L13_1(L14_1, L15_1)
