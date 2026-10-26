local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1
L0_1 = "20170927.26"
L1_1 = "-[ JSON.lua package by Jeffrey Friedl (http://regex.info/blog/lua/json) version 20170927.26 ]-"
L2_1 = {}
L2_1.VERSION = L0_1
L2_1.AUTHOR_NOTE = L1_1
L3_1 = "  "
L4_1 = {}
L4_1.pretty = true
L4_1.indent = L3_1
L4_1.align_keys = false
L4_1.array_newline = false
L5_1 = {}

function L6_1()
  local L0_2, L1_2
  L0_2 = "JSON array"
  return L0_2
end

L5_1.__tostring = L6_1
L5_1.__index = L5_1
L6_1 = {}

function L7_1()
  local L0_2, L1_2
  L0_2 = "JSON object"
  return L0_2
end

L6_1.__tostring = L7_1
L6_1.__index = L6_1

function L7_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = setmetatable
  L3_2 = A1_2 or L3_2
  if not A1_2 then
    L3_2 = {}
  end
  L4_2 = L5_1
  return L2_2(L3_2, L4_2)
end

L2_1.newArray = L7_1

function L7_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = setmetatable
  L3_2 = A1_2 or L3_2
  if not A1_2 then
    L3_2 = {}
  end
  L4_2 = L6_1
  return L2_2(L3_2, L4_2)
end

L2_1.newObject = L7_1

function L7_1(A0_2)
  local L1_2, L2_2
  L1_2 = type
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L1_2 = A0_2 or L1_2
  if L1_2 ~= "number" or not A0_2 then
    L1_2 = A0_2.N
  end
  return L1_2
end

L8_1 = {}

function L9_1(A0_2)
  local L1_2
  L1_2 = A0_2.S
  return L1_2
end

L8_1.__tostring = L9_1

function L9_1(A0_2)
  local L1_2, L2_2
  L1_2 = L7_1
  L2_2 = A0_2
  return L1_2(L2_2)
end

L8_1.__unm = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = tostring
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = tostring
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 .. L3_2
  return L2_2
end

L8_1.__concat = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 + L3_2
  return L2_2
end

L8_1.__add = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 - L3_2
  return L2_2
end

L8_1.__sub = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 * L3_2
  return L2_2
end

L8_1.__mul = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 / L3_2
  return L2_2
end

L8_1.__div = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 % L3_2
  return L2_2
end

L8_1.__mod = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 ^ L3_2
  return L2_2
end

L8_1.__pow = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 < L3_2
  return L2_2
end

L8_1.__lt = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 == L3_2
  return L2_2
end

L8_1.__eq = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = L7_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 <= L3_2
  return L2_2
end

L8_1.__le = L9_1
L8_1.__index = L8_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = getmetatable
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  L3_2 = L8_1
  if L2_2 == L3_2 then
    return A1_2
  else
    L2_2 = type
    L3_2 = A1_2
    L2_2 = L2_2(L3_2)
    if L2_2 == "table" then
      L2_2 = type
      L3_2 = A1_2.S
      L2_2 = L2_2(L3_2)
      if L2_2 == "string" then
        L2_2 = type
        L3_2 = A1_2.N
        L2_2 = L2_2(L3_2)
        if L2_2 == "number" then
          L2_2 = setmetatable
          L3_2 = A1_2
          L4_2 = L8_1
          return L2_2(L3_2, L4_2)
      end
    end
    else
      L2_2 = {}
      L3_2 = tostring
      L4_2 = A1_2
      L3_2 = L3_2(L4_2)
      L2_2.S = L3_2
      L3_2 = tonumber
      L4_2 = A1_2
      L3_2 = L3_2(L4_2)
      L2_2.N = L3_2
      L3_2 = setmetatable
      L4_2 = L2_2
      L5_2 = L8_1
      return L3_2(L4_2, L5_2)
    end
  end
end

L2_1.asNumber = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = type
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "table" then
    L2_2 = type
    L3_2 = A1_2.S
    L2_2 = L2_2(L3_2)
    if L2_2 == "string" then
      L2_2 = A1_2.S
      return L2_2
  end
  else
    L2_2 = tostring
    L3_2 = A1_2
    return L2_2(L3_2)
  end
end

L2_1.forceString = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = type
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "table" then
    L2_2 = type
    L3_2 = A1_2.N
    L2_2 = L2_2(L3_2)
    if L2_2 == "number" then
      L2_2 = A1_2.N
      return L2_2
  end
  else
    L2_2 = tonumber
    L3_2 = A1_2
    return L2_2(L3_2)
  end
end

L2_1.forceNumber = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = type
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "number" then
    return A1_2
  else
    L2_2 = type
    L3_2 = A1_2
    L2_2 = L2_2(L3_2)
    if L2_2 == "table" then
      L2_2 = type
      L3_2 = A1_2.N
      L2_2 = L2_2(L3_2)
      if L2_2 == "number" then
        L2_2 = A1_2.N
        return L2_2
    end
    else
      L2_2 = nil
      return L2_2
    end
  end
end

L2_1.isNumber = L9_1

function L9_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = type
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "string" then
    return A1_2
  else
    L2_2 = type
    L3_2 = A1_2
    L2_2 = L2_2(L3_2)
    if L2_2 == "table" then
      L2_2 = type
      L3_2 = A1_2.S
      L2_2 = L2_2(L3_2)
      if L2_2 == "string" then
        L2_2 = A1_2.S
        return L2_2
    end
    else
      L2_2 = nil
      return L2_2
    end
  end
end

L2_1.isString = L9_1

function L9_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  if A0_2 <= 127 then
    L1_2 = string
    L1_2 = L1_2.char
    L2_2 = A0_2
    return L1_2(L2_2)
  elseif A0_2 <= 2047 then
    L1_2 = math
    L1_2 = L1_2.floor
    L2_2 = A0_2 / 64
    L1_2 = L1_2(L2_2)
    L2_2 = 64 * L1_2
    L2_2 = A0_2 - L2_2
    L3_2 = string
    L3_2 = L3_2.char
    L4_2 = 192 + L1_2
    L5_2 = 128 + L2_2
    return L3_2(L4_2, L5_2)
  elseif A0_2 <= 65535 then
    L1_2 = math
    L1_2 = L1_2.floor
    L2_2 = A0_2 / 4096
    L1_2 = L1_2(L2_2)
    L2_2 = 4096 * L1_2
    L2_2 = A0_2 - L2_2
    L3_2 = math
    L3_2 = L3_2.floor
    L4_2 = L2_2 / 64
    L3_2 = L3_2(L4_2)
    L4_2 = 64 * L3_2
    L4_2 = L2_2 - L4_2
    L1_2 = 224 + L1_2
    L3_2 = 128 + L3_2
    L4_2 = 128 + L4_2
    if L1_2 == 224 and L3_2 < 160 or L1_2 == 237 and 159 < L3_2 or L1_2 == 240 and L3_2 < 144 or L1_2 == 244 and 143 < L3_2 then
      L5_2 = "?"
      return L5_2
    else
      L5_2 = string
      L5_2 = L5_2.char
      L6_2 = L1_2
      L7_2 = L3_2
      L8_2 = L4_2
      return L5_2(L6_2, L7_2, L8_2)
    end
  else
    L1_2 = math
    L1_2 = L1_2.floor
    L2_2 = A0_2 / 262144
    L1_2 = L1_2(L2_2)
    L2_2 = 262144 * L1_2
    L2_2 = A0_2 - L2_2
    L3_2 = math
    L3_2 = L3_2.floor
    L4_2 = L2_2 / 4096
    L3_2 = L3_2(L4_2)
    L4_2 = 4096 * L3_2
    L2_2 = L2_2 - L4_2
    L4_2 = math
    L4_2 = L4_2.floor
    L5_2 = L2_2 / 64
    L4_2 = L4_2(L5_2)
    L5_2 = 64 * L4_2
    L5_2 = L2_2 - L5_2
    L6_2 = string
    L6_2 = L6_2.char
    L7_2 = 240 + L1_2
    L8_2 = 128 + L3_2
    L9_2 = 128 + L4_2
    L10_2 = 128 + L5_2
    return L6_2(L7_2, L8_2, L9_2, L10_2)
  end
end

function L10_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2
  if A2_2 then
    if A3_2 then
      L5_2 = string
      L5_2 = L5_2.format
      L6_2 = "%s at byte %d of: %s"
      L7_2 = A1_2
      L8_2 = A3_2
      L9_2 = A2_2
      L5_2 = L5_2(L6_2, L7_2, L8_2, L9_2)
      A1_2 = L5_2
    else
      L5_2 = string
      L5_2 = L5_2.format
      L6_2 = "%s: %s"
      L7_2 = A1_2
      L8_2 = A2_2
      L5_2 = L5_2(L6_2, L7_2, L8_2)
      A1_2 = L5_2
    end
  end
  if A4_2 ~= nil then
    L5_2 = A1_2
    L6_2 = " ("
    L7_2 = L2_1
    L8_2 = L7_2
    L7_2 = L7_2.encode
    L9_2 = A4_2
    L7_2 = L7_2(L8_2, L9_2)
    L8_2 = ")"
    A1_2 = L5_2 .. L6_2 .. L7_2 .. L8_2
  end
  L5_2 = A0_2.assert
  if L5_2 then
    L5_2 = A0_2.assert
    L6_2 = false
    L7_2 = A1_2
    L5_2(L6_2, L7_2)
  else
    L5_2 = assert
    L6_2 = false
    L7_2 = A1_2
    L5_2(L6_2, L7_2)
  end
end

L2_1.onDecodeError = L10_1

function L10_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L6_2 = A0_2
  L5_2 = A0_2.onDecodeError
  L7_2 = "trailing garbage"
  L8_2 = A1_2
  L9_2 = A2_2
  L10_2 = A4_2
  return L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
end

L2_1.onTrailingGarbage = L10_1
L10_1 = L2_1.onDecodeError
L2_1.onDecodeOfNilError = L10_1
L10_1 = L2_1.onDecodeError
L2_1.onDecodeOfHTMLError = L10_1

function L10_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2
  if A2_2 ~= nil then
    L3_2 = A1_2
    L4_2 = " ("
    L5_2 = L2_1
    L6_2 = L5_2
    L5_2 = L5_2.encode
    L7_2 = A2_2
    L5_2 = L5_2(L6_2, L7_2)
    L6_2 = ")"
    A1_2 = L3_2 .. L4_2 .. L5_2 .. L6_2
  end
  L3_2 = A0_2.assert
  if L3_2 then
    L3_2 = A0_2.assert
    L4_2 = false
    L5_2 = A1_2
    L3_2(L4_2, L5_2)
  else
    L3_2 = assert
    L4_2 = false
    L5_2 = A1_2
    L3_2(L4_2, L5_2)
  end
end

L2_1.onEncodeError = L10_1

function L10_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2
  L5_2 = A1_2
  L4_2 = A1_2.match
  L6_2 = "^-?[1-9]%d*"
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if not L4_2 then
    L5_2 = A1_2
    L4_2 = A1_2.match
    L6_2 = "^-?0"
    L7_2 = A2_2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
  end
  if not L4_2 then
    L6_2 = A0_2
    L5_2 = A0_2.onDecodeError
    L7_2 = "expected number"
    L8_2 = A1_2
    L9_2 = A2_2
    L10_2 = A3_2.etc
    L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
    L5_2 = nil
    L6_2 = A2_2
    return L5_2, L6_2
  end
  L6_2 = L4_2
  L5_2 = L4_2.len
  L5_2 = L5_2(L6_2)
  L5_2 = A2_2 + L5_2
  L7_2 = A1_2
  L6_2 = A1_2.match
  L8_2 = "^%.%d+"
  L9_2 = L5_2
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  if not L6_2 then
    L6_2 = ""
  end
  L8_2 = L6_2
  L7_2 = L6_2.len
  L7_2 = L7_2(L8_2)
  L5_2 = L5_2 + L7_2
  L8_2 = A1_2
  L7_2 = A1_2.match
  L9_2 = "^[eE][-+]?%d+"
  L10_2 = L5_2
  L7_2 = L7_2(L8_2, L9_2, L10_2)
  if not L7_2 then
    L7_2 = ""
  end
  L9_2 = L7_2
  L8_2 = L7_2.len
  L8_2 = L8_2(L9_2)
  L5_2 = L5_2 + L8_2
  L8_2 = L4_2
  L9_2 = L6_2
  L10_2 = L7_2
  L8_2 = L8_2 .. L9_2 .. L10_2
  L9_2 = A3_2.decodeNumbersAsObjects
  if L9_2 then
    L9_2 = false
    L10_2 = A3_2.decodeIntegerObjectificationLength
    if not L10_2 then
      L10_2 = A3_2.decodeDecimalObjectificationLength
      if not L10_2 then
        L9_2 = true
    end
    else
      L10_2 = A3_2.decodeIntegerObjectificationLength
      if L10_2 then
        L11_2 = L4_2
        L10_2 = L4_2.len
        L10_2 = L10_2(L11_2)
        L11_2 = A3_2.decodeIntegerObjectificationLength
        if L10_2 >= L11_2 then
          goto lbl_85
        end
        L11_2 = L7_2
        L10_2 = L7_2.len
        L10_2 = L10_2(L11_2)
        if 0 < L10_2 then
          goto lbl_85
        end
      end
      L10_2 = A3_2.decodeDecimalObjectificationLength
      if L10_2 then
        L11_2 = L6_2
        L10_2 = L6_2.len
        L10_2 = L10_2(L11_2)
        L11_2 = A3_2.decodeDecimalObjectificationLength
        if not (L10_2 >= L11_2) then
          L11_2 = L7_2
          L10_2 = L7_2.len
          L10_2 = L10_2(L11_2)
          if not (0 < L10_2) then
            goto lbl_86
          end
        end
        ::lbl_85::
        L9_2 = true
      end
    end
    ::lbl_86::
    if L9_2 then
      L10_2 = L2_1
      L11_2 = L10_2
      L10_2 = L10_2.asNumber
      L12_2 = L8_2
      L10_2 = L10_2(L11_2, L12_2)
      L11_2 = L5_2
      return L10_2, L11_2
    end
  else
    L9_2 = A3_2.decodeIntegerStringificationLength
    if L9_2 then
      L10_2 = L4_2
      L9_2 = L4_2.len
      L9_2 = L9_2(L10_2)
      L10_2 = A3_2.decodeIntegerStringificationLength
      if L9_2 >= L10_2 then
        goto lbl_119
      end
      L10_2 = L7_2
      L9_2 = L7_2.len
      L9_2 = L9_2(L10_2)
      if 0 < L9_2 then
        goto lbl_119
      end
    end
    L9_2 = A3_2.decodeDecimalStringificationLength
    if L9_2 then
      L10_2 = L6_2
      L9_2 = L6_2.len
      L9_2 = L9_2(L10_2)
      L10_2 = A3_2.decodeDecimalStringificationLength
      if not (L9_2 >= L10_2) then
        L10_2 = L7_2
        L9_2 = L7_2.len
        L9_2 = L9_2(L10_2)
        if not (0 < L9_2) then
          goto lbl_122
        end
      end
      ::lbl_119::
      L9_2 = L8_2
      L10_2 = L5_2
      return L9_2, L10_2
    end
  end
  ::lbl_122::
  L9_2 = tonumber
  L10_2 = L8_2
  L9_2 = L9_2(L10_2)
  if not L9_2 then
    L11_2 = A0_2
    L10_2 = A0_2.onDecodeError
    L12_2 = "bad number"
    L13_2 = A1_2
    L14_2 = A2_2
    L15_2 = A3_2.etc
    L10_2(L11_2, L12_2, L13_2, L14_2, L15_2)
    L10_2 = nil
    L11_2 = A2_2
    return L10_2, L11_2
  end
  L10_2 = L9_2
  L11_2 = L5_2
  return L10_2, L11_2
end

function L11_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L5_2 = A1_2
  L4_2 = A1_2.sub
  L6_2 = A2_2
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if L4_2 ~= "\"" then
    L5_2 = A0_2
    L4_2 = A0_2.onDecodeError
    L6_2 = "expected string's opening quote"
    L7_2 = A1_2
    L8_2 = A2_2
    L9_2 = A3_2.etc
    L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
    L4_2 = nil
    L5_2 = A2_2
    return L4_2, L5_2
  end
  L4_2 = A2_2 + 1
  L6_2 = A1_2
  L5_2 = A1_2.len
  L5_2 = L5_2(L6_2)
  L6_2 = ""
  while L4_2 <= L5_2 do
    L8_2 = A1_2
    L7_2 = A1_2.sub
    L9_2 = L4_2
    L10_2 = L4_2
    L7_2 = L7_2(L8_2, L9_2, L10_2)
    if L7_2 == "\"" then
      L8_2 = L6_2
      L9_2 = L4_2 + 1
      return L8_2, L9_2
    end
    if L7_2 ~= "\\" then
      L8_2 = L6_2
      L9_2 = L7_2
      L6_2 = L8_2 .. L9_2
      L4_2 = L4_2 + 1
    else
      L9_2 = A1_2
      L8_2 = A1_2.match
      L10_2 = "^\\b"
      L11_2 = L4_2
      L8_2 = L8_2(L9_2, L10_2, L11_2)
      if L8_2 then
        L8_2 = L6_2
        L9_2 = "\b"
        L6_2 = L8_2 .. L9_2
        L4_2 = L4_2 + 2
      else
        L9_2 = A1_2
        L8_2 = A1_2.match
        L10_2 = "^\\f"
        L11_2 = L4_2
        L8_2 = L8_2(L9_2, L10_2, L11_2)
        if L8_2 then
          L8_2 = L6_2
          L9_2 = "\f"
          L6_2 = L8_2 .. L9_2
          L4_2 = L4_2 + 2
        else
          L9_2 = A1_2
          L8_2 = A1_2.match
          L10_2 = "^\\n"
          L11_2 = L4_2
          L8_2 = L8_2(L9_2, L10_2, L11_2)
          if L8_2 then
            L8_2 = L6_2
            L9_2 = "\n"
            L6_2 = L8_2 .. L9_2
            L4_2 = L4_2 + 2
          else
            L9_2 = A1_2
            L8_2 = A1_2.match
            L10_2 = "^\\r"
            L11_2 = L4_2
            L8_2 = L8_2(L9_2, L10_2, L11_2)
            if L8_2 then
              L8_2 = L6_2
              L9_2 = "\r"
              L6_2 = L8_2 .. L9_2
              L4_2 = L4_2 + 2
            else
              L9_2 = A1_2
              L8_2 = A1_2.match
              L10_2 = "^\\t"
              L11_2 = L4_2
              L8_2 = L8_2(L9_2, L10_2, L11_2)
              if L8_2 then
                L8_2 = L6_2
                L9_2 = "\t"
                L6_2 = L8_2 .. L9_2
                L4_2 = L4_2 + 2
              else
                L9_2 = A1_2
                L8_2 = A1_2.match
                L10_2 = "^\\u([0123456789aAbBcCdDeEfF][0123456789aAbBcCdDeEfF][0123456789aAbBcCdDeEfF][0123456789aAbBcCdDeEfF])"
                L11_2 = L4_2
                L8_2 = L8_2(L9_2, L10_2, L11_2)
                if L8_2 then
                  L4_2 = L4_2 + 6
                  L9_2 = tonumber
                  L10_2 = L8_2
                  L11_2 = 16
                  L9_2 = L9_2(L10_2, L11_2)
                  if 55296 <= L9_2 and L9_2 <= 56319 then
                    L11_2 = A1_2
                    L10_2 = A1_2.match
                    L12_2 = "^\\u([dD][cdefCDEF][0123456789aAbBcCdDeEfF][0123456789aAbBcCdDeEfF])"
                    L13_2 = L4_2
                    L10_2 = L10_2(L11_2, L12_2, L13_2)
                    if L10_2 then
                      L4_2 = L4_2 + 6
                      L11_2 = L9_2 - 55296
                      L11_2 = L11_2 * 1024
                      L11_2 = 9216 + L11_2
                      L12_2 = tonumber
                      L13_2 = L10_2
                      L14_2 = 16
                      L12_2 = L12_2(L13_2, L14_2)
                      L9_2 = L11_2 + L12_2
                    else
                    end
                  end
                  L10_2 = L6_2
                  L11_2 = L9_1
                  L12_2 = L9_2
                  L11_2 = L11_2(L12_2)
                  L6_2 = L10_2 .. L11_2
                else
                  L9_2 = L6_2
                  L11_2 = A1_2
                  L10_2 = A1_2.match
                  L12_2 = "^\\(.)"
                  L13_2 = L4_2
                  L10_2 = L10_2(L11_2, L12_2, L13_2)
                  L6_2 = L9_2 .. L10_2
                  L4_2 = L4_2 + 2
                end
              end
            end
          end
        end
      end
    end
  end
  L8_2 = A0_2
  L7_2 = A0_2.onDecodeError
  L9_2 = "unclosed string"
  L10_2 = A1_2
  L11_2 = A2_2
  L12_2 = A3_2.etc
  L7_2(L8_2, L9_2, L10_2, L11_2, L12_2)
  L7_2 = nil
  L8_2 = A2_2
  return L7_2, L8_2
end

function L12_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L3_2 = A0_2
  L2_2 = A0_2.find
  L4_2 = "^[ \n\r\t]+"
  L5_2 = A1_2
  L2_2, L3_2 = L2_2(L3_2, L4_2, L5_2)
  if L3_2 then
    L4_2 = L3_2 + 1
    return L4_2
  else
    return A1_2
  end
end

L13_1 = nil

function L14_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2
  L5_2 = A1_2
  L4_2 = A1_2.sub
  L6_2 = A2_2
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if L4_2 ~= "{" then
    L5_2 = A0_2
    L4_2 = A0_2.onDecodeError
    L6_2 = "expected '{'"
    L7_2 = A1_2
    L8_2 = A2_2
    L9_2 = A3_2.etc
    L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
    L4_2 = nil
    L5_2 = A2_2
    return L4_2, L5_2
  end
  L4_2 = L12_1
  L5_2 = A1_2
  L6_2 = A2_2 + 1
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = A0_2.strictTypes
  if L5_2 then
    L6_2 = A0_2
    L5_2 = A0_2.newObject
    L7_2 = {}
    L5_2 = L5_2(L6_2, L7_2)
    if L5_2 then
      goto lbl_29
    end
  end
  L5_2 = {}
  ::lbl_29::
  L7_2 = A1_2
  L6_2 = A1_2.sub
  L8_2 = L4_2
  L9_2 = L4_2
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  if L6_2 == "}" then
    L6_2 = L5_2
    L7_2 = L4_2 + 1
    return L6_2, L7_2
  end
  L7_2 = A1_2
  L6_2 = A1_2.len
  L6_2 = L6_2(L7_2)
  while L4_2 <= L6_2 do
    L7_2 = L11_1
    L8_2 = A0_2
    L9_2 = A1_2
    L10_2 = L4_2
    L11_2 = A3_2
    L7_2, L8_2 = L7_2(L8_2, L9_2, L10_2, L11_2)
    L9_2 = L12_1
    L10_2 = A1_2
    L11_2 = L8_2
    L9_2 = L9_2(L10_2, L11_2)
    L4_2 = L9_2
    L10_2 = A1_2
    L9_2 = A1_2.sub
    L11_2 = L4_2
    L12_2 = L4_2
    L9_2 = L9_2(L10_2, L11_2, L12_2)
    if L9_2 ~= ":" then
      L10_2 = A0_2
      L9_2 = A0_2.onDecodeError
      L11_2 = "expected colon"
      L12_2 = A1_2
      L13_2 = L4_2
      L14_2 = A3_2.etc
      L9_2(L10_2, L11_2, L12_2, L13_2, L14_2)
      L9_2 = nil
      L10_2 = L4_2
      return L9_2, L10_2
    end
    L9_2 = L12_1
    L10_2 = A1_2
    L11_2 = L4_2 + 1
    L9_2 = L9_2(L10_2, L11_2)
    L4_2 = L9_2
    L9_2 = L13_1
    L10_2 = A0_2
    L11_2 = A1_2
    L12_2 = L4_2
    L13_2 = A3_2
    L9_2, L10_2 = L9_2(L10_2, L11_2, L12_2, L13_2)
    L5_2[L7_2] = L9_2
    L11_2 = L12_1
    L12_2 = A1_2
    L13_2 = L10_2
    L11_2 = L11_2(L12_2, L13_2)
    L4_2 = L11_2
    L12_2 = A1_2
    L11_2 = A1_2.sub
    L13_2 = L4_2
    L14_2 = L4_2
    L11_2 = L11_2(L12_2, L13_2, L14_2)
    if L11_2 == "}" then
      L12_2 = L5_2
      L13_2 = L4_2 + 1
      return L12_2, L13_2
    end
    L13_2 = A1_2
    L12_2 = A1_2.sub
    L14_2 = L4_2
    L15_2 = L4_2
    L12_2 = L12_2(L13_2, L14_2, L15_2)
    if L12_2 ~= "," then
      L13_2 = A0_2
      L12_2 = A0_2.onDecodeError
      L14_2 = "expected comma or '}'"
      L15_2 = A1_2
      L16_2 = L4_2
      L17_2 = A3_2.etc
      L12_2(L13_2, L14_2, L15_2, L16_2, L17_2)
      L12_2 = nil
      L13_2 = L4_2
      return L12_2, L13_2
    end
    L12_2 = L12_1
    L13_2 = A1_2
    L14_2 = L4_2 + 1
    L12_2 = L12_2(L13_2, L14_2)
    L4_2 = L12_2
  end
  L8_2 = A0_2
  L7_2 = A0_2.onDecodeError
  L9_2 = "unclosed '{'"
  L10_2 = A1_2
  L11_2 = A2_2
  L12_2 = A3_2.etc
  L7_2(L8_2, L9_2, L10_2, L11_2, L12_2)
  L7_2 = nil
  L8_2 = A2_2
  return L7_2, L8_2
end

function L15_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L5_2 = A1_2
  L4_2 = A1_2.sub
  L6_2 = A2_2
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if L4_2 ~= "[" then
    L5_2 = A0_2
    L4_2 = A0_2.onDecodeError
    L6_2 = "expected '['"
    L7_2 = A1_2
    L8_2 = A2_2
    L9_2 = A3_2.etc
    L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
    L4_2 = nil
    L5_2 = A2_2
    return L4_2, L5_2
  end
  L4_2 = L12_1
  L5_2 = A1_2
  L6_2 = A2_2 + 1
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = A0_2.strictTypes
  if L5_2 then
    L6_2 = A0_2
    L5_2 = A0_2.newArray
    L7_2 = {}
    L5_2 = L5_2(L6_2, L7_2)
    if L5_2 then
      goto lbl_29
    end
  end
  L5_2 = {}
  ::lbl_29::
  L7_2 = A1_2
  L6_2 = A1_2.sub
  L8_2 = L4_2
  L9_2 = L4_2
  L6_2 = L6_2(L7_2, L8_2, L9_2)
  if L6_2 == "]" then
    L6_2 = L5_2
    L7_2 = L4_2 + 1
    return L6_2, L7_2
  end
  L6_2 = 1
  L8_2 = A1_2
  L7_2 = A1_2.len
  L7_2 = L7_2(L8_2)
  while L4_2 <= L7_2 do
    L8_2 = L13_1
    L9_2 = A0_2
    L10_2 = A1_2
    L11_2 = L4_2
    L12_2 = A3_2
    L8_2, L9_2 = L8_2(L9_2, L10_2, L11_2, L12_2)
    L5_2[L6_2] = L8_2
    L6_2 = L6_2 + 1
    L10_2 = L12_1
    L11_2 = A1_2
    L12_2 = L9_2
    L10_2 = L10_2(L11_2, L12_2)
    L4_2 = L10_2
    L11_2 = A1_2
    L10_2 = A1_2.sub
    L12_2 = L4_2
    L13_2 = L4_2
    L10_2 = L10_2(L11_2, L12_2, L13_2)
    if L10_2 == "]" then
      L11_2 = L5_2
      L12_2 = L4_2 + 1
      return L11_2, L12_2
    end
    L12_2 = A1_2
    L11_2 = A1_2.sub
    L13_2 = L4_2
    L14_2 = L4_2
    L11_2 = L11_2(L12_2, L13_2, L14_2)
    if L11_2 ~= "," then
      L12_2 = A0_2
      L11_2 = A0_2.onDecodeError
      L13_2 = "expected comma or ']'"
      L14_2 = A1_2
      L15_2 = L4_2
      L16_2 = A3_2.etc
      L11_2(L12_2, L13_2, L14_2, L15_2, L16_2)
      L11_2 = nil
      L12_2 = L4_2
      return L11_2, L12_2
    end
    L11_2 = L12_1
    L12_2 = A1_2
    L13_2 = L4_2 + 1
    L11_2 = L11_2(L12_2, L13_2)
    L4_2 = L11_2
  end
  L9_2 = A0_2
  L8_2 = A0_2.onDecodeError
  L10_2 = "unclosed '['"
  L11_2 = A1_2
  L12_2 = A2_2
  L13_2 = A3_2.etc
  L8_2(L9_2, L10_2, L11_2, L12_2, L13_2)
  L8_2 = nil
  L9_2 = L4_2
  return L8_2, L9_2
end

function L13_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L4_2 = L12_1
  L5_2 = A1_2
  L6_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2)
  A2_2 = L4_2
  L5_2 = A1_2
  L4_2 = A1_2.len
  L4_2 = L4_2(L5_2)
  if A2_2 > L4_2 then
    L5_2 = A0_2
    L4_2 = A0_2.onDecodeError
    L6_2 = "unexpected end of string"
    L7_2 = A1_2
    L8_2 = nil
    L9_2 = A3_2.etc
    L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
    L4_2 = nil
    L5_2 = A2_2
    return L4_2, L5_2
  end
  L5_2 = A1_2
  L4_2 = A1_2.find
  L6_2 = "^\""
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if L4_2 then
    L4_2 = L11_1
    L5_2 = A0_2
    L6_2 = A1_2
    L7_2 = A2_2
    L8_2 = A3_2.etc
    return L4_2(L5_2, L6_2, L7_2, L8_2)
  else
    L5_2 = A1_2
    L4_2 = A1_2.find
    L6_2 = "^[-0123456789 ]"
    L7_2 = A2_2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    if L4_2 then
      L4_2 = L10_1
      L5_2 = A0_2
      L6_2 = A1_2
      L7_2 = A2_2
      L8_2 = A3_2
      return L4_2(L5_2, L6_2, L7_2, L8_2)
    else
      L5_2 = A1_2
      L4_2 = A1_2.find
      L6_2 = "^%{"
      L7_2 = A2_2
      L4_2 = L4_2(L5_2, L6_2, L7_2)
      if L4_2 then
        L4_2 = L14_1
        L5_2 = A0_2
        L6_2 = A1_2
        L7_2 = A2_2
        L8_2 = A3_2
        return L4_2(L5_2, L6_2, L7_2, L8_2)
      else
        L5_2 = A1_2
        L4_2 = A1_2.find
        L6_2 = "^%["
        L7_2 = A2_2
        L4_2 = L4_2(L5_2, L6_2, L7_2)
        if L4_2 then
          L4_2 = L15_1
          L5_2 = A0_2
          L6_2 = A1_2
          L7_2 = A2_2
          L8_2 = A3_2
          return L4_2(L5_2, L6_2, L7_2, L8_2)
        else
          L5_2 = A1_2
          L4_2 = A1_2.find
          L6_2 = "^true"
          L7_2 = A2_2
          L4_2 = L4_2(L5_2, L6_2, L7_2)
          if L4_2 then
            L4_2 = true
            L5_2 = A2_2 + 4
            return L4_2, L5_2
          else
            L5_2 = A1_2
            L4_2 = A1_2.find
            L6_2 = "^false"
            L7_2 = A2_2
            L4_2 = L4_2(L5_2, L6_2, L7_2)
            if L4_2 then
              L4_2 = false
              L5_2 = A2_2 + 5
              return L4_2, L5_2
            else
              L5_2 = A1_2
              L4_2 = A1_2.find
              L6_2 = "^null"
              L7_2 = A2_2
              L4_2 = L4_2(L5_2, L6_2, L7_2)
              if L4_2 then
                L4_2 = A3_2.null
                L5_2 = A2_2 + 4
                return L4_2, L5_2
              else
                L5_2 = A0_2
                L4_2 = A0_2.onDecodeError
                L6_2 = "can't parse JSON"
                L7_2 = A1_2
                L8_2 = A2_2
                L9_2 = A3_2.etc
                L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
                L4_2 = nil
                L5_2 = 1
                return L4_2, L5_2
              end
            end
          end
        end
      end
    end
  end
end

function L16_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L4_2 = type
  L5_2 = A3_2
  L4_2 = L4_2(L5_2)
  if L4_2 ~= "table" then
    L4_2 = {}
    A3_2 = L4_2
  end
  if A2_2 ~= nil then
    A3_2.etc = A2_2
  end
  L4_2 = type
  L5_2 = A0_2
  L4_2 = L4_2(L5_2)
  if L4_2 == "table" then
    L4_2 = A0_2.__index
    L5_2 = L2_1
    if L4_2 == L5_2 then
      goto lbl_30
    end
  end
  L4_2 = "JSON:decode must be called in method format"
  L5_2 = L2_1
  L6_2 = L5_2
  L5_2 = L5_2.onDecodeError
  L7_2 = L4_2
  L8_2 = nil
  L9_2 = nil
  L10_2 = A3_2.etc
  L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
  L5_2 = nil
  L6_2 = L4_2
  do return L5_2, L6_2 end
  ::lbl_30::
  if A1_2 == nil then
    L4_2 = "nil passed to JSON:decode()"
    L6_2 = A0_2
    L5_2 = A0_2.onDecodeOfNilError
    L7_2 = L4_2
    L8_2 = nil
    L9_2 = nil
    L10_2 = A3_2.etc
    L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
    L5_2 = nil
    L6_2 = L4_2
    return L5_2, L6_2
  else
    L4_2 = type
    L5_2 = A1_2
    L4_2 = L4_2(L5_2)
    if L4_2 ~= "string" then
      L4_2 = "expected string argument to JSON:decode()"
      L6_2 = A0_2
      L5_2 = A0_2.onDecodeError
      L7_2 = string
      L7_2 = L7_2.format
      L8_2 = "%s, got %s"
      L9_2 = L4_2
      L10_2 = type
      L11_2 = A1_2
      L10_2, L11_2, L12_2, L13_2 = L10_2(L11_2)
      L7_2 = L7_2(L8_2, L9_2, L10_2, L11_2, L12_2, L13_2)
      L8_2 = nil
      L9_2 = nil
      L10_2 = A3_2.etc
      L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
      L5_2 = nil
      L6_2 = L4_2
      return L5_2, L6_2
    end
  end
  L5_2 = A1_2
  L4_2 = A1_2.match
  L6_2 = "^%s*$"
  L4_2 = L4_2(L5_2, L6_2)
  if L4_2 then
    L4_2 = nil
    return L4_2
  end
  L5_2 = A1_2
  L4_2 = A1_2.match
  L6_2 = "^%s*<"
  L4_2 = L4_2(L5_2, L6_2)
  if L4_2 then
    L4_2 = "HTML passed to JSON:decode()"
    L6_2 = A0_2
    L5_2 = A0_2.onDecodeOfHTMLError
    L7_2 = L4_2
    L8_2 = A1_2
    L9_2 = nil
    L10_2 = A3_2.etc
    L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
    L5_2 = nil
    L6_2 = L4_2
    return L5_2, L6_2
  end
  L5_2 = A1_2
  L4_2 = A1_2.sub
  L6_2 = 1
  L7_2 = 1
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  L5_2 = L4_2
  L4_2 = L4_2.byte
  L4_2 = L4_2(L5_2)
  if L4_2 ~= 0 then
    L5_2 = A1_2
    L4_2 = A1_2.len
    L4_2 = L4_2(L5_2)
    if not (2 <= L4_2) then
      goto lbl_115
    end
    L5_2 = A1_2
    L4_2 = A1_2.sub
    L6_2 = 2
    L7_2 = 2
    L4_2 = L4_2(L5_2, L6_2, L7_2)
    L5_2 = L4_2
    L4_2 = L4_2.byte
    L4_2 = L4_2(L5_2)
    if L4_2 ~= 0 then
      goto lbl_115
    end
  end
  L4_2 = "JSON package groks only UTF-8, sorry"
  L6_2 = A0_2
  L5_2 = A0_2.onDecodeError
  L7_2 = L4_2
  L8_2 = A1_2
  L9_2 = nil
  L10_2 = A3_2.etc
  L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
  L5_2 = nil
  L6_2 = L4_2
  do return L5_2, L6_2 end
  ::lbl_115::
  L4_2 = A3_2.decodeNumbersAsObjects
  if L4_2 == nil then
    L4_2 = A0_2.decodeNumbersAsObjects
    A3_2.decodeNumbersAsObjects = L4_2
  end
  L4_2 = A3_2.decodeIntegerObjectificationLength
  if L4_2 == nil then
    L4_2 = A0_2.decodeIntegerObjectificationLength
    A3_2.decodeIntegerObjectificationLength = L4_2
  end
  L4_2 = A3_2.decodeDecimalObjectificationLength
  if L4_2 == nil then
    L4_2 = A0_2.decodeDecimalObjectificationLength
    A3_2.decodeDecimalObjectificationLength = L4_2
  end
  L4_2 = A3_2.decodeIntegerStringificationLength
  if L4_2 == nil then
    L4_2 = A0_2.decodeIntegerStringificationLength
    A3_2.decodeIntegerStringificationLength = L4_2
  end
  L4_2 = A3_2.decodeDecimalStringificationLength
  if L4_2 == nil then
    L4_2 = A0_2.decodeDecimalStringificationLength
    A3_2.decodeDecimalStringificationLength = L4_2
  end
  L4_2 = pcall
  L5_2 = L13_1
  L6_2 = A0_2
  L7_2 = A1_2
  L8_2 = 1
  L9_2 = A3_2
  L4_2, L5_2, L6_2 = L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
  if L4_2 then
    L7_2 = nil
    L8_2 = #A1_2
    L8_2 = L8_2 + 1
    if L6_2 ~= L8_2 then
      L8_2 = L12_1
      L9_2 = A1_2
      L10_2 = L6_2
      L8_2 = L8_2(L9_2, L10_2)
      L6_2 = L8_2
      L8_2 = #A1_2
      L8_2 = L8_2 + 1
      if L6_2 ~= L8_2 then
        L9_2 = A0_2
        L8_2 = A0_2.onTrailingGarbage
        L10_2 = A1_2
        L11_2 = L6_2
        L12_2 = L5_2
        L13_2 = A3_2.etc
        L8_2, L9_2 = L8_2(L9_2, L10_2, L11_2, L12_2, L13_2)
        L7_2 = L9_2
        L5_2 = L8_2
      end
    end
    L8_2 = L5_2
    L9_2 = L7_2
    return L8_2, L9_2
  else
    L7_2 = L5_2
    L8_2 = A0_2.assert
    if L8_2 then
      L8_2 = A0_2.assert
      L9_2 = false
      L10_2 = L7_2
      L8_2(L9_2, L10_2)
    else
      L8_2 = assert
      L9_2 = false
      L10_2 = L7_2
      L8_2(L9_2, L10_2)
    end
    L8_2 = nil
    L9_2 = L7_2
    return L8_2, L9_2
  end
end

L2_1.decode = L16_1

function L16_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  if A0_2 == "\n" then
    L1_2 = "\\n"
    return L1_2
  elseif A0_2 == "\r" then
    L1_2 = "\\r"
    return L1_2
  elseif A0_2 == "\t" then
    L1_2 = "\\t"
    return L1_2
  elseif A0_2 == "\b" then
    L1_2 = "\\b"
    return L1_2
  elseif A0_2 == "\f" then
    L1_2 = "\\f"
    return L1_2
  elseif A0_2 == "\"" then
    L1_2 = "\\\""
    return L1_2
  elseif A0_2 == "\\" then
    L1_2 = "\\\\"
    return L1_2
  else
    L1_2 = string
    L1_2 = L1_2.format
    L2_2 = "\\u%04x"
    L4_2 = A0_2
    L3_2 = A0_2.byte
    L3_2, L4_2 = L3_2(L4_2)
    return L1_2(L2_2, L3_2, L4_2)
  end
end

L17_1 = "["
L18_1 = "\""
L19_1 = "%\\"
L20_1 = "%z"
L21_1 = "\001"
L22_1 = "-"
L23_1 = "\031"
L24_1 = "]"
L17_1 = L17_1 .. L18_1 .. L19_1 .. L20_1 .. L21_1 .. L22_1 .. L23_1 .. L24_1
L18_1 = L9_1
L19_1 = 8232
L18_1 = L18_1(L19_1)
L19_1 = L9_1
L20_1 = 8233
L19_1 = L19_1(L20_1)

function L20_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L3_2 = A0_2
  L2_2 = A0_2.gsub
  L4_2 = L17_1
  L5_2 = L16_1
  L2_2 = L2_2(L3_2, L4_2, L5_2)
  L3_2 = A1_2.stringsAreUtf8
  if L3_2 then
    L4_2 = L2_2
    L3_2 = L2_2.gsub
    L5_2 = L18_1
    L6_2 = "\\u2028"
    L3_2 = L3_2(L4_2, L5_2, L6_2)
    L4_2 = L3_2
    L3_2 = L3_2.gsub
    L5_2 = L19_1
    L6_2 = "\\u2029"
    L3_2 = L3_2(L4_2, L5_2, L6_2)
    L2_2 = L3_2
  end
  L3_2 = "\""
  L4_2 = L2_2
  L5_2 = "\""
  L3_2 = L3_2 .. L4_2 .. L5_2
  return L3_2
end

function L21_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2
  L3_2 = {}
  L4_2 = {}
  L5_2 = false
  L6_2 = nil
  L7_2 = pairs
  L8_2 = A1_2
  L7_2, L8_2, L9_2 = L7_2(L8_2)
  for L10_2 in L7_2, L8_2, L9_2 do
    L11_2 = type
    L12_2 = L10_2
    L11_2 = L11_2(L12_2)
    if L11_2 == "string" then
      L11_2 = table
      L11_2 = L11_2.insert
      L12_2 = L3_2
      L13_2 = L10_2
      L11_2(L12_2, L13_2)
    else
      L11_2 = type
      L12_2 = L10_2
      L11_2 = L11_2(L12_2)
      if L11_2 == "number" then
        L11_2 = table
        L11_2 = L11_2.insert
        L12_2 = L4_2
        L13_2 = L10_2
        L11_2(L12_2, L13_2)
        if not (L10_2 <= 0) then
          L11_2 = math
          L11_2 = L11_2.huge
          if not (L10_2 >= L11_2) then
            goto lbl_38
          end
        end
        L5_2 = true
        goto lbl_65
        ::lbl_38::
        if not L6_2 or L10_2 > L6_2 then
          L6_2 = L10_2
        end
      else
        L11_2 = type
        L12_2 = L10_2
        L11_2 = L11_2(L12_2)
        if L11_2 == "boolean" then
          L11_2 = table
          L11_2 = L11_2.insert
          L12_2 = L3_2
          L13_2 = tostring
          L14_2 = L10_2
          L13_2, L14_2, L15_2, L16_2, L17_2, L18_2 = L13_2(L14_2)
          L11_2(L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2)
        else
          L12_2 = A0_2
          L11_2 = A0_2.onEncodeError
          L13_2 = "can't encode table with a key of type "
          L14_2 = type
          L15_2 = L10_2
          L14_2 = L14_2(L15_2)
          L13_2 = L13_2 .. L14_2
          L14_2 = A2_2
          L11_2(L12_2, L13_2, L14_2)
        end
      end
    end
    ::lbl_65::
  end
  L7_2 = #L3_2
  if L7_2 == 0 and not L5_2 then
    L7_2 = #L4_2
    if 0 < L7_2 then
      L7_2 = nil
      L8_2 = L6_2
      return L7_2, L8_2
    else
      L7_2 = tostring
      L8_2 = A1_2
      L7_2 = L7_2(L8_2)
      if L7_2 == "JSON array" then
        L7_2 = nil
        return L7_2
      else
        L7_2 = tostring
        L8_2 = A1_2
        L7_2 = L7_2(L8_2)
        if L7_2 == "JSON object" then
          L7_2 = {}
          return L7_2
        else
          L7_2 = nil
          return L7_2
        end
      end
    end
  end
  L7_2 = table
  L7_2 = L7_2.sort
  L8_2 = L3_2
  L7_2(L8_2)
  L7_2 = nil
  L8_2 = #L4_2
  if 0 < L8_2 then
    L8_2 = A0_2.noKeyConversion
    if L8_2 then
      L9_2 = A0_2
      L8_2 = A0_2.onEncodeError
      L10_2 = "a table with both numeric and string keys could be an object or array; aborting"
      L11_2 = A2_2
      L8_2(L9_2, L10_2, L11_2)
    end
    L8_2 = {}
    L7_2 = L8_2
    L8_2 = pairs
    L9_2 = A1_2
    L8_2, L9_2, L10_2 = L8_2(L9_2)
    for L11_2, L12_2 in L8_2, L9_2, L10_2 do
      L7_2[L11_2] = L12_2
    end
    L8_2 = table
    L8_2 = L8_2.sort
    L9_2 = L4_2
    L8_2(L9_2)
    L8_2 = ipairs
    L9_2 = L4_2
    L8_2, L9_2, L10_2 = L8_2(L9_2)
    for L11_2, L12_2 in L8_2, L9_2, L10_2 do
      L13_2 = tostring
      L14_2 = L12_2
      L13_2 = L13_2(L14_2)
      L14_2 = L7_2[L13_2]
      if L14_2 == nil then
        L14_2 = table
        L14_2 = L14_2.insert
        L15_2 = L3_2
        L16_2 = L13_2
        L14_2(L15_2, L16_2)
        L14_2 = A1_2[L12_2]
        L7_2[L13_2] = L14_2
      else
        L15_2 = A0_2
        L14_2 = A0_2.onEncodeError
        L16_2 = "conflict converting table with mixed-type keys into a JSON object: key "
        L17_2 = L12_2
        L18_2 = " exists both as a string and a number."
        L16_2 = L16_2 .. L17_2 .. L18_2
        L17_2 = A2_2
        L14_2(L15_2, L16_2, L17_2)
      end
    end
  end
  L8_2 = L3_2
  L9_2 = nil
  L10_2 = L7_2
  return L8_2, L9_2, L10_2
end

function L22_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2, A6_2)
  local L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2
  if A1_2 ~= nil then
    if not (not A6_2 and A4_2) then
      goto lbl_16
    end
    L7_2 = A4_2.null
    if not L7_2 then
      goto lbl_16
    end
    L7_2 = A4_2.null
    if A1_2 ~= L7_2 then
      goto lbl_16
    end
  end
  L7_2 = "null"
  do return L7_2 end
  goto lbl_392
  ::lbl_16::
  L7_2 = type
  L8_2 = A1_2
  L7_2 = L7_2(L8_2)
  if L7_2 == "string" then
    L7_2 = L20_1
    L8_2 = A1_2
    L9_2 = A4_2
    return L7_2(L8_2, L9_2)
  else
    L7_2 = type
    L8_2 = A1_2
    L7_2 = L7_2(L8_2)
    if L7_2 == "number" then
      if A1_2 ~= A1_2 then
        L7_2 = "null"
        return L7_2
      else
        L7_2 = math
        L7_2 = L7_2.huge
        if A1_2 >= L7_2 then
          L7_2 = "1e+9999"
          return L7_2
        else
          L7_2 = math
          L7_2 = L7_2.huge
          L7_2 = -L7_2
          if A1_2 <= L7_2 then
            L7_2 = "-1e+9999"
            return L7_2
          else
            L7_2 = tostring
            L8_2 = A1_2
            return L7_2(L8_2)
          end
        end
      end
    else
      L7_2 = type
      L8_2 = A1_2
      L7_2 = L7_2(L8_2)
      if L7_2 == "boolean" then
        L7_2 = tostring
        L8_2 = A1_2
        return L7_2(L8_2)
      else
        L7_2 = type
        L8_2 = A1_2
        L7_2 = L7_2(L8_2)
        if L7_2 ~= "table" then
          L7_2 = A0_2.unsupportedTypeEncoder
          if L7_2 then
            L8_2 = A0_2
            L7_2 = A0_2.unsupportedTypeEncoder
            L9_2 = A1_2
            L10_2 = A2_2
            L11_2 = A3_2
            L12_2 = A4_2
            L13_2 = A5_2
            L14_2 = A6_2
            L7_2, L8_2 = L7_2(L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2)
            L9_2 = type
            L10_2 = L7_2
            L9_2 = L9_2(L10_2)
            if L9_2 == "string" then
              return L7_2
            elseif L7_2 ~= nil then
              L10_2 = A0_2
              L9_2 = A0_2.onEncodeError
              L11_2 = "unsupportedTypeEncoder method returned a "
              L12_2 = type
              L13_2 = L7_2
              L12_2 = L12_2(L13_2)
              L11_2 = L11_2 .. L12_2
              L12_2 = A3_2
              L9_2(L10_2, L11_2, L12_2)
            elseif L8_2 then
              L10_2 = A0_2
              L9_2 = A0_2.onEncodeError
              L11_2 = tostring
              L12_2 = L8_2
              L11_2 = L11_2(L12_2)
              L12_2 = A3_2
              L9_2(L10_2, L11_2, L12_2)
            end
          end
          L8_2 = A0_2
          L7_2 = A0_2.onEncodeError
          L9_2 = "can't convert "
          L10_2 = type
          L11_2 = A1_2
          L10_2 = L10_2(L11_2)
          L11_2 = " to JSON"
          L9_2 = L9_2 .. L10_2 .. L11_2
          L10_2 = A3_2
          L7_2(L8_2, L9_2, L10_2)
        else
          L7_2 = getmetatable
          L8_2 = A1_2
          L7_2 = L7_2(L8_2)
          L8_2 = L8_1
          if L7_2 == L8_2 then
            L7_2 = tostring
            L8_2 = A1_2
            return L7_2(L8_2)
          else
            L7_2 = A1_2
            L8_2 = type
            L9_2 = A4_2
            L8_2 = L8_2(L9_2)
            if L8_2 ~= "table" then
              L8_2 = {}
              A4_2 = L8_2
            end
            L8_2 = type
            L9_2 = A5_2
            L8_2 = L8_2(L9_2)
            if L8_2 ~= "string" then
              A5_2 = ""
            end
            L8_2 = A2_2[L7_2]
            if L8_2 then
              L9_2 = A0_2
              L8_2 = A0_2.onEncodeError
              L10_2 = "table "
              L11_2 = tostring
              L12_2 = L7_2
              L11_2 = L11_2(L12_2)
              L12_2 = " is a child of itself"
              L10_2 = L10_2 .. L11_2 .. L12_2
              L11_2 = A3_2
              L8_2(L9_2, L10_2, L11_2)
            else
              A2_2[L7_2] = true
            end
            L8_2 = nil
            L9_2 = L21_1
            L10_2 = A0_2
            L11_2 = L7_2
            L12_2 = A3_2
            L9_2, L10_2, L11_2 = L9_2(L10_2, L11_2, L12_2)
            if L10_2 then
              L12_2 = nil
              L13_2 = A4_2.array_newline
              if L13_2 then
                L13_2 = A5_2
                L14_2 = tostring
                L15_2 = A4_2.indent
                if not L15_2 then
                  L15_2 = ""
                end
                L14_2 = L14_2(L15_2)
                L12_2 = L13_2 .. L14_2
              else
                L12_2 = A5_2
              end
              L13_2 = {}
              L14_2 = 1
              L15_2 = L10_2
              L16_2 = 1
              for L17_2 = L14_2, L15_2, L16_2 do
                L18_2 = table
                L18_2 = L18_2.insert
                L19_2 = L13_2
                L20_2 = L22_1
                L21_2 = A0_2
                L22_2 = L7_2[L17_2]
                L23_2 = A2_2
                L24_2 = A3_2
                L25_2 = A4_2
                L26_2 = L12_2
                L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2 = L20_2(L21_2, L22_2, L23_2, L24_2, L25_2, L26_2)
                L18_2(L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2)
              end
              L14_2 = A4_2.array_newline
              if L14_2 then
                L14_2 = "[\n"
                L15_2 = L12_2
                L16_2 = table
                L16_2 = L16_2.concat
                L17_2 = L13_2
                L18_2 = ",\n"
                L19_2 = L12_2
                L18_2 = L18_2 .. L19_2
                L16_2 = L16_2(L17_2, L18_2)
                L17_2 = "\n"
                L18_2 = A5_2
                L19_2 = "]"
                L8_2 = L14_2 .. L15_2 .. L16_2 .. L17_2 .. L18_2 .. L19_2
              else
                L14_2 = A4_2.pretty
                if L14_2 then
                  L14_2 = "[ "
                  L15_2 = table
                  L15_2 = L15_2.concat
                  L16_2 = L13_2
                  L17_2 = ", "
                  L15_2 = L15_2(L16_2, L17_2)
                  L16_2 = " ]"
                  L8_2 = L14_2 .. L15_2 .. L16_2
                else
                  L14_2 = "["
                  L15_2 = table
                  L15_2 = L15_2.concat
                  L16_2 = L13_2
                  L17_2 = ","
                  L15_2 = L15_2(L16_2, L17_2)
                  L16_2 = "]"
                  L8_2 = L14_2 .. L15_2 .. L16_2
                end
              end
            elseif L9_2 then
              L12_2 = L11_2 or L12_2
              if not L11_2 then
                L12_2 = L7_2
              end
              L13_2 = A4_2.pretty
              if L13_2 then
                L13_2 = {}
                L14_2 = 0
                L15_2 = ipairs
                L16_2 = L9_2
                L15_2, L16_2, L17_2 = L15_2(L16_2)
                for L18_2, L19_2 in L15_2, L16_2, L17_2 do
                  L20_2 = L22_1
                  L21_2 = A0_2
                  L22_2 = tostring
                  L23_2 = L19_2
                  L22_2 = L22_2(L23_2)
                  L23_2 = A2_2
                  L24_2 = A3_2
                  L25_2 = A4_2
                  L26_2 = A5_2
                  L27_2 = true
                  L20_2 = L20_2(L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2)
                  L21_2 = A4_2.align_keys
                  if L21_2 then
                    L21_2 = math
                    L21_2 = L21_2.max
                    L22_2 = L14_2
                    L23_2 = #L20_2
                    L21_2 = L21_2(L22_2, L23_2)
                    L14_2 = L21_2
                  end
                  L21_2 = table
                  L21_2 = L21_2.insert
                  L22_2 = L13_2
                  L23_2 = L20_2
                  L21_2(L22_2, L23_2)
                end
                L15_2 = A5_2
                L16_2 = tostring
                L17_2 = A4_2.indent
                if not L17_2 then
                  L17_2 = ""
                end
                L16_2 = L16_2(L17_2)
                L15_2 = L15_2 .. L16_2
                L16_2 = L15_2
                L17_2 = string
                L17_2 = L17_2.rep
                L18_2 = " "
                L19_2 = L14_2
                L17_2 = L17_2(L18_2, L19_2)
                L18_2 = A4_2.align_keys
                if L18_2 then
                  L18_2 = "  "
                  if L18_2 then
                    goto lbl_298
                  end
                end
                L18_2 = ""
                ::lbl_298::
                L16_2 = L16_2 .. L17_2 .. L18_2
                L17_2 = "%s%"
                L18_2 = string
                L18_2 = L18_2.format
                L19_2 = "%d"
                L20_2 = L14_2
                L18_2 = L18_2(L19_2, L20_2)
                L19_2 = "s: %s"
                L17_2 = L17_2 .. L18_2 .. L19_2
                L18_2 = {}
                L19_2 = ipairs
                L20_2 = L9_2
                L19_2, L20_2, L21_2 = L19_2(L20_2)
                for L22_2, L23_2 in L19_2, L20_2, L21_2 do
                  L24_2 = L22_1
                  L25_2 = A0_2
                  L26_2 = L12_2[L23_2]
                  L27_2 = A2_2
                  L28_2 = A3_2
                  L29_2 = A4_2
                  L30_2 = L16_2
                  L24_2 = L24_2(L25_2, L26_2, L27_2, L28_2, L29_2, L30_2)
                  L25_2 = table
                  L25_2 = L25_2.insert
                  L26_2 = L18_2
                  L27_2 = string
                  L27_2 = L27_2.format
                  L28_2 = L17_2
                  L29_2 = L15_2
                  L30_2 = L13_2[L22_2]
                  L31_2 = L24_2
                  L27_2, L28_2, L29_2, L30_2, L31_2 = L27_2(L28_2, L29_2, L30_2, L31_2)
                  L25_2(L26_2, L27_2, L28_2, L29_2, L30_2, L31_2)
                end
                L19_2 = "{\n"
                L20_2 = table
                L20_2 = L20_2.concat
                L21_2 = L18_2
                L22_2 = ",\n"
                L20_2 = L20_2(L21_2, L22_2)
                L21_2 = "\n"
                L22_2 = A5_2
                L23_2 = "}"
                L8_2 = L19_2 .. L20_2 .. L21_2 .. L22_2 .. L23_2
              else
                L13_2 = {}
                L14_2 = ipairs
                L15_2 = L9_2
                L14_2, L15_2, L16_2 = L14_2(L15_2)
                for L17_2, L18_2 in L14_2, L15_2, L16_2 do
                  L19_2 = L22_1
                  L20_2 = A0_2
                  L21_2 = L12_2[L18_2]
                  L22_2 = A2_2
                  L23_2 = A3_2
                  L24_2 = A4_2
                  L25_2 = A5_2
                  L19_2 = L19_2(L20_2, L21_2, L22_2, L23_2, L24_2, L25_2)
                  L20_2 = L22_1
                  L21_2 = A0_2
                  L22_2 = tostring
                  L23_2 = L18_2
                  L22_2 = L22_2(L23_2)
                  L23_2 = A2_2
                  L24_2 = A3_2
                  L25_2 = A4_2
                  L26_2 = A5_2
                  L27_2 = true
                  L20_2 = L20_2(L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2)
                  L21_2 = table
                  L21_2 = L21_2.insert
                  L22_2 = L13_2
                  L23_2 = string
                  L23_2 = L23_2.format
                  L24_2 = "%s:%s"
                  L25_2 = L20_2
                  L26_2 = L19_2
                  L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2 = L23_2(L24_2, L25_2, L26_2)
                  L21_2(L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2)
                end
                L14_2 = "{"
                L15_2 = table
                L15_2 = L15_2.concat
                L16_2 = L13_2
                L17_2 = ","
                L15_2 = L15_2(L16_2, L17_2)
                L16_2 = "}"
                L8_2 = L14_2 .. L15_2 .. L16_2
              end
            else
              L8_2 = "[]"
            end
            A2_2[L7_2] = false
            return L8_2
          end
        end
      end
    end
  end
  ::lbl_392::
end

function L23_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L4_2 = L22_1
  L5_2 = A0_2
  L6_2 = A1_2
  L7_2 = {}
  L8_2 = A2_2
  L9_2 = A3_2
  L4_2 = L4_2(L5_2, L6_2, L7_2, L8_2, L9_2)
  if L4_2 == nil then
    return L4_2
  else
    return L4_2
  end
end

function L24_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2
  L4_2 = type
  L5_2 = A0_2
  L4_2 = L4_2(L5_2)
  if L4_2 == "table" then
    L4_2 = A0_2.__index
    L5_2 = L2_1
    if L4_2 == L5_2 then
      goto lbl_15
    end
  end
  L4_2 = L2_1
  L5_2 = L4_2
  L4_2 = L4_2.onEncodeError
  L6_2 = "JSON:encode must be called in method format"
  L7_2 = A2_2
  L4_2(L5_2, L6_2, L7_2)
  ::lbl_15::
  L4_2 = type
  L5_2 = A3_2
  L4_2 = L4_2(L5_2)
  if L4_2 ~= "table" then
    L4_2 = {}
    A3_2 = L4_2
  end
  L4_2 = L23_1
  L5_2 = A0_2
  L6_2 = A1_2
  L7_2 = A2_2
  L8_2 = A3_2
  return L4_2(L5_2, L6_2, L7_2, L8_2)
end

L2_1.encode = L24_1

function L24_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2
  L4_2 = type
  L5_2 = A0_2
  L4_2 = L4_2(L5_2)
  if L4_2 == "table" then
    L4_2 = A0_2.__index
    L5_2 = L2_1
    if L4_2 == L5_2 then
      goto lbl_15
    end
  end
  L4_2 = L2_1
  L5_2 = L4_2
  L4_2 = L4_2.onEncodeError
  L6_2 = "JSON:encode_pretty must be called in method format"
  L7_2 = A2_2
  L4_2(L5_2, L6_2, L7_2)
  ::lbl_15::
  L4_2 = type
  L5_2 = A3_2
  L4_2 = L4_2(L5_2)
  if L4_2 ~= "table" then
    A3_2 = L4_1
  end
  L4_2 = L23_1
  L5_2 = A0_2
  L6_2 = A1_2
  L7_2 = A2_2
  L8_2 = A3_2
  return L4_2(L5_2, L6_2, L7_2, L8_2)
end

L2_1.encode_pretty = L24_1

function L24_1()
  local L0_2, L1_2
  L0_2 = "JSON encode/decode package"
  return L0_2
end

L2_1.__tostring = L24_1
L2_1.__index = L2_1

function L24_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = {}
  if A1_2 then
    L3_2 = pairs
    L4_2 = A1_2
    L3_2, L4_2, L5_2 = L3_2(L4_2)
    for L6_2, L7_2 in L3_2, L4_2, L5_2 do
      L2_2[L6_2] = L7_2
    end
  end
  L3_2 = setmetatable
  L4_2 = L2_2
  L5_2 = L2_1
  return L3_2(L4_2, L5_2)
end

L2_1.new = L24_1
L25_1 = L2_1
L24_1 = L2_1.new
return L24_1(L25_1)
