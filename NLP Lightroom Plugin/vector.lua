local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1
L0_1 = assert
L1_1 = math
L1_1 = L1_1.sqrt
L2_1 = math
L2_1 = L2_1.cos
L3_1 = math
L3_1 = L3_1.sin
L4_1 = math
L4_1 = L4_1.atan2
L5_1 = {}
L5_1.__index = L5_1

function L6_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = setmetatable
  L3_2 = {}
  L4_2 = A0_2 or L4_2
  if not A0_2 then
    L4_2 = 0
  end
  L3_2.x = L4_2
  L4_2 = A1_2 or L4_2
  if not A1_2 then
    L4_2 = 0
  end
  L3_2.y = L4_2
  L4_2 = L5_1
  return L2_2(L3_2, L4_2)
end

L7_1 = L6_1
L8_1 = 0
L9_1 = 0
L7_1 = L7_1(L8_1, L9_1)

function L8_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  if not A1_2 then
    A1_2 = 1
  end
  L2_2 = L6_1
  L3_2 = L2_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  L3_2 = L3_2 * A1_2
  L4_2 = L3_1
  L5_2 = A0_2
  L4_2 = L4_2(L5_2)
  L4_2 = L4_2 * A1_2
  return L2_2(L3_2, L4_2)
end

function L9_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  if not A0_2 then
    A0_2 = 1
  end
  if not A1_2 then
    A1_2 = A0_2
  end
  L2_2 = L0_1
  L3_2 = 0 < A1_2
  L4_2 = "len_max must be greater than zero"
  L2_2(L3_2, L4_2)
  L2_2 = L0_1
  L3_2 = A0_2 <= A1_2
  L4_2 = "len_max must be greater than or equal to len_min"
  L2_2(L3_2, L4_2)
  L2_2 = L8_1
  L3_2 = math
  L3_2 = L3_2.random
  L3_2 = L3_2()
  L3_2 = L3_2 * 2
  L4_2 = math
  L4_2 = L4_2.pi
  L3_2 = L3_2 * L4_2
  L4_2 = math
  L4_2 = L4_2.random
  L4_2 = L4_2()
  L5_2 = A1_2 - A0_2
  L4_2 = L4_2 * L5_2
  L4_2 = L4_2 + A0_2
  return L2_2(L3_2, L4_2)
end

function L10_1(A0_2)
  local L1_2, L2_2
  L1_2 = type
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L1_2 = L1_2 == "table"
  return L1_2
end

function L11_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L6_1
  L2_2 = A0_2.x
  L3_2 = A0_2.y
  return L1_2(L2_2, L3_2)
end

L5_1.clone = L11_1

function L11_1(A0_2)
  local L1_2, L2_2
  L1_2 = A0_2.x
  L2_2 = A0_2.y
  return L1_2, L2_2
end

L5_1.unpack = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = "("
  L2_2 = tonumber
  L3_2 = A0_2.x
  L2_2 = L2_2(L3_2)
  L3_2 = ","
  L4_2 = tonumber
  L5_2 = A0_2.y
  L4_2 = L4_2(L5_2)
  L5_2 = ")"
  L1_2 = L1_2 .. L2_2 .. L3_2 .. L4_2 .. L5_2
  return L1_2
end

L5_1.__tostring = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L6_1
  L2_2 = A0_2.x
  L2_2 = -L2_2
  L3_2 = A0_2.y
  L3_2 = -L3_2
  return L1_2(L2_2, L3_2)
end

L5_1.__unm = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L10_1
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
  end
  L4_2 = "Add: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = L6_1
  L3_2 = A0_2.x
  L4_2 = A1_2.x
  L3_2 = L3_2 + L4_2
  L4_2 = A0_2.y
  L5_2 = A1_2.y
  L4_2 = L4_2 + L5_2
  return L2_2(L3_2, L4_2)
end

L5_1.__add = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L10_1
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
  end
  L4_2 = "Sub: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = L6_1
  L3_2 = A0_2.x
  L4_2 = A1_2.x
  L3_2 = L3_2 - L4_2
  L4_2 = A0_2.y
  L5_2 = A1_2.y
  L4_2 = L4_2 - L5_2
  return L2_2(L3_2, L4_2)
end

L5_1.__sub = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = type
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "number" then
    L2_2 = L6_1
    L3_2 = A1_2.x
    L3_2 = A0_2 * L3_2
    L4_2 = A1_2.y
    L4_2 = A0_2 * L4_2
    return L2_2(L3_2, L4_2)
  else
    L2_2 = type
    L3_2 = A1_2
    L2_2 = L2_2(L3_2)
    if L2_2 == "number" then
      L2_2 = L6_1
      L3_2 = A0_2.x
      L3_2 = A1_2 * L3_2
      L4_2 = A0_2.y
      L4_2 = A1_2 * L4_2
      return L2_2(L3_2, L4_2)
    else
      L2_2 = L0_1
      L3_2 = L10_1
      L4_2 = A0_2
      L3_2 = L3_2(L4_2)
      if L3_2 then
        L3_2 = L10_1
        L4_2 = A1_2
        L3_2 = L3_2(L4_2)
      end
      L4_2 = "Mul: wrong argument types (<vector> or <number> expected)"
      L2_2(L3_2, L4_2)
      L2_2 = A0_2.x
      L3_2 = A1_2.x
      L2_2 = L2_2 * L3_2
      L3_2 = A0_2.y
      L4_2 = A1_2.y
      L3_2 = L3_2 * L4_2
      L2_2 = L2_2 + L3_2
      return L2_2
    end
  end
end

L5_1.__mul = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = type
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
    L3_2 = L3_2 == "number"
  end
  L4_2 = "wrong argument types (expected <vector> / <number>)"
  L2_2(L3_2, L4_2)
  L2_2 = L6_1
  L3_2 = A0_2.x
  L3_2 = L3_2 / A1_2
  L4_2 = A0_2.y
  L4_2 = L4_2 / A1_2
  return L2_2(L3_2, L4_2)
end

L5_1.__div = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 == L3_2
  return L2_2
end

L5_1.__eq = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 < L3_2
  return L2_2
end

L5_1.__lt = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 <= L3_2
  return L2_2
end

L5_1.__le = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L10_1
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
  end
  L4_2 = "permul: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = L6_1
  L3_2 = A0_2.x
  L4_2 = A1_2.x
  L3_2 = L3_2 * L4_2
  L4_2 = A0_2.y
  L5_2 = A1_2.y
  L4_2 = L4_2 * L5_2
  return L2_2(L3_2, L4_2)
end

L5_1.permul = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = L6_1
  L2_2 = L4_1
  L3_2 = A0_2.x
  L4_2 = A0_2.y
  L2_2 = L2_2(L3_2, L4_2)
  L4_2 = A0_2
  L3_2 = A0_2.len
  L3_2, L4_2 = L3_2(L4_2)
  return L1_2(L2_2, L3_2, L4_2)
end

L5_1.toPolar = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = A0_2.x
  L2_2 = A0_2.x
  L1_2 = L1_2 * L2_2
  L2_2 = A0_2.y
  L3_2 = A0_2.y
  L2_2 = L2_2 * L3_2
  L1_2 = L1_2 + L2_2
  return L1_2
end

L5_1.len2 = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = L1_1
  L2_2 = A0_2.x
  L3_2 = A0_2.x
  L2_2 = L2_2 * L3_2
  L3_2 = A0_2.y
  L4_2 = A0_2.y
  L3_2 = L3_2 * L4_2
  L2_2 = L2_2 + L3_2
  return L1_2(L2_2)
end

L5_1.len = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L10_1
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
  end
  L4_2 = "dist: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 - L3_2
  L3_2 = A0_2.y
  L4_2 = A1_2.y
  L3_2 = L3_2 - L4_2
  L4_2 = L1_1
  L5_2 = L2_2 * L2_2
  L6_2 = L3_2 * L3_2
  L5_2 = L5_2 + L6_2
  return L4_2(L5_2)
end

L5_1.dist = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L10_1
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
  end
  L4_2 = "dist: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 - L3_2
  L3_2 = A0_2.y
  L4_2 = A1_2.y
  L3_2 = L3_2 - L4_2
  L4_2 = L2_2 * L2_2
  L5_2 = L3_2 * L3_2
  L4_2 = L4_2 + L5_2
  return L4_2
end

L5_1.dist2 = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2
  L2_2 = A0_2
  L1_2 = A0_2.len
  L1_2 = L1_2(L2_2)
  if 0 < L1_2 then
    L2_2 = A0_2.x
    L2_2 = L2_2 / L1_2
    L3_2 = A0_2.y
    L3_2 = L3_2 / L1_2
    A0_2.y = L3_2
    A0_2.x = L2_2
  end
  return A0_2
end

L5_1.normalizeInplace = L11_1

function L11_1(A0_2)
  local L1_2, L2_2
  L2_2 = A0_2
  L1_2 = A0_2.clone
  L1_2 = L1_2(L2_2)
  L2_2 = L1_2
  L1_2 = L1_2.normalizeInplace
  return L1_2(L2_2)
end

L5_1.normalized = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L2_1
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  L3_2 = L3_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = A0_2.x
  L4_2 = L2_2 * L4_2
  L5_2 = A0_2.y
  L5_2 = L3_2 * L5_2
  L4_2 = L4_2 - L5_2
  L5_2 = A0_2.x
  L5_2 = L3_2 * L5_2
  L6_2 = A0_2.y
  L6_2 = L2_2 * L6_2
  L5_2 = L5_2 + L6_2
  A0_2.y = L5_2
  A0_2.x = L4_2
  return A0_2
end

L5_1.rotateInplace = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = L2_1
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  L3_2 = L3_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = L6_1
  L5_2 = A0_2.x
  L5_2 = L2_2 * L5_2
  L6_2 = A0_2.y
  L6_2 = L3_2 * L6_2
  L5_2 = L5_2 - L6_2
  L6_2 = A0_2.x
  L6_2 = L3_2 * L6_2
  L7_2 = A0_2.y
  L7_2 = L2_2 * L7_2
  L6_2 = L6_2 + L7_2
  return L4_2(L5_2, L6_2)
end

L5_1.rotated = L11_1

function L11_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L6_1
  L2_2 = A0_2.y
  L2_2 = -L2_2
  L3_2 = A0_2.x
  return L1_2(L2_2, L3_2)
end

L5_1.perpendicular = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = "invalid argument: cannot project vector on "
  L5_2 = type
  L6_2 = A1_2
  L5_2 = L5_2(L6_2)
  L4_2 = L4_2 .. L5_2
  L2_2(L3_2, L4_2)
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 * L3_2
  L3_2 = A0_2.y
  L4_2 = A1_2.y
  L3_2 = L3_2 * L4_2
  L2_2 = L2_2 + L3_2
  L3_2 = A1_2.x
  L4_2 = A1_2.x
  L3_2 = L3_2 * L4_2
  L4_2 = A1_2.y
  L5_2 = A1_2.y
  L4_2 = L4_2 * L5_2
  L3_2 = L3_2 + L4_2
  L2_2 = L2_2 / L3_2
  L3_2 = L6_1
  L4_2 = A1_2.x
  L4_2 = L2_2 * L4_2
  L5_2 = A1_2.y
  L5_2 = L2_2 * L5_2
  return L3_2(L4_2, L5_2)
end

L5_1.projectOn = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = "invalid argument: cannot mirror vector on "
  L5_2 = type
  L6_2 = A1_2
  L5_2 = L5_2(L6_2)
  L4_2 = L4_2 .. L5_2
  L2_2(L3_2, L4_2)
  L2_2 = A0_2.x
  L3_2 = A1_2.x
  L2_2 = L2_2 * L3_2
  L3_2 = A0_2.y
  L4_2 = A1_2.y
  L3_2 = L3_2 * L4_2
  L2_2 = L2_2 + L3_2
  L2_2 = 2 * L2_2
  L3_2 = A1_2.x
  L4_2 = A1_2.x
  L3_2 = L3_2 * L4_2
  L4_2 = A1_2.y
  L5_2 = A1_2.y
  L4_2 = L4_2 * L5_2
  L3_2 = L3_2 + L4_2
  L2_2 = L2_2 / L3_2
  L3_2 = L6_1
  L4_2 = A1_2.x
  L4_2 = L2_2 * L4_2
  L5_2 = A0_2.x
  L4_2 = L4_2 - L5_2
  L5_2 = A1_2.y
  L5_2 = L2_2 * L5_2
  L6_2 = A0_2.y
  L5_2 = L5_2 - L6_2
  return L3_2(L4_2, L5_2)
end

L5_1.mirrorOn = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L0_1
  L3_2 = L10_1
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = "cross: wrong argument types (<vector> expected)"
  L2_2(L3_2, L4_2)
  L2_2 = A0_2.x
  L3_2 = A1_2.y
  L2_2 = L2_2 * L3_2
  L3_2 = A0_2.y
  L4_2 = A1_2.x
  L3_2 = L3_2 * L4_2
  L2_2 = L2_2 - L3_2
  return L2_2
end

L5_1.cross = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = A1_2 * A1_2
  L4_2 = A0_2
  L3_2 = A0_2.len2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 / L3_2
  if 1 < L2_2 then
    L3_2 = 1
    if L3_2 then
      goto lbl_15
      L2_2 = L3_2 or L2_2
    end
  end
  L3_2 = math
  L3_2 = L3_2.sqrt
  L4_2 = L2_2
  L3_2 = L3_2(L4_2)
  L2_2 = L3_2
  ::lbl_15::
  L3_2 = A0_2.x
  L3_2 = L3_2 * L2_2
  L4_2 = A0_2.y
  L4_2 = L4_2 * L2_2
  A0_2.y = L4_2
  A0_2.x = L3_2
  return A0_2
end

L5_1.trimInplace = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  if A1_2 then
    L2_2 = L4_1
    L3_2 = A0_2.y
    L4_2 = A0_2.x
    L2_2 = L2_2(L3_2, L4_2)
    L3_2 = L4_1
    L4_2 = A1_2.y
    L5_2 = A1_2.x
    L3_2 = L3_2(L4_2, L5_2)
    L2_2 = L2_2 - L3_2
    return L2_2
  end
  L2_2 = L4_1
  L3_2 = A0_2.y
  L4_2 = A0_2.x
  return L2_2(L3_2, L4_2)
end

L5_1.angleTo = L11_1

function L11_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L3_2 = A0_2
  L2_2 = A0_2.clone
  L2_2 = L2_2(L3_2)
  L3_2 = L2_2
  L2_2 = L2_2.trimInplace
  L4_2 = A1_2
  return L2_2(L3_2, L4_2)
end

L5_1.trimmed = L11_1
L11_1 = setmetatable
L12_1 = {}
L12_1.new = L6_1
L12_1.fromPolar = L8_1
L12_1.randomDirection = L9_1
L12_1.isvector = L10_1
L12_1.zero = L7_1
L13_1 = {}

function L14_1(A0_2, ...)
  local L2_2, L3_2
  L2_2 = L6_1
  L3_2 = ...
  return L2_2(L3_2)
end

L13_1.__call = L14_1
return L11_1(L12_1, L13_1)
