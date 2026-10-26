local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1
L0_1 = {}
L0_1._VERSION = "inspect.lua 3.1.0"
L0_1._URL = "http://github.com/kikito/inspect.lua"
L0_1._DESCRIPTION = "human-readable representations of tables"
L0_1._LICENSE = "    MIT LICENSE\n\n    Copyright (c) 2013 Enrique Garc\195\173a Cota\n\n    Permission is hereby granted, free of charge, to any person obtaining a\n    copy of this software and associated documentation files (the\n    \"Software\"), to deal in the Software without restriction, including\n    without limitation the rights to use, copy, modify, merge, publish,\n    distribute, sublicense, and/or sell copies of the Software, and to\n    permit persons to whom the Software is furnished to do so, subject to\n    the following conditions:\n\n    The above copyright notice and this permission notice shall be included\n    in all copies or substantial portions of the Software.\n\n    THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS\n    OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF\n    MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.\n    IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY\n    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,\n    TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE\n    SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.\n  "
L1_1 = tostring
L2_1 = setmetatable
L3_1 = {}
L4_1 = {}

function L5_1()
  local L0_2, L1_2
  L0_2 = "inspect.KEY"
  return L0_2
end

L4_1.__tostring = L5_1
L2_1 = L2_1(L3_1, L4_1)
L0_1.KEY = L2_1
L2_1 = setmetatable
L3_1 = {}
L4_1 = {}

function L5_1()
  local L0_2, L1_2
  L0_2 = "inspect.METATABLE"
  return L0_2
end

L4_1.__tostring = L5_1
L2_1 = L2_1(L3_1, L4_1)
L0_1.METATABLE = L2_1

function L2_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = next
  L2_2 = A0_2
  L3_2 = nil
  return L1_2, L2_2, L3_2
end

function L3_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L2_2 = A0_2
  L1_2 = A0_2.match
  L3_2 = "\""
  L1_2 = L1_2(L2_2, L3_2)
  if L1_2 then
    L2_2 = A0_2
    L1_2 = A0_2.match
    L3_2 = "'"
    L1_2 = L1_2(L2_2, L3_2)
    if not L1_2 then
      L1_2 = "'"
      L2_2 = A0_2
      L3_2 = "'"
      L1_2 = L1_2 .. L2_2 .. L3_2
      return L1_2
    end
  end
  L1_2 = "\""
  L3_2 = A0_2
  L2_2 = A0_2.gsub
  L4_2 = "\""
  L5_2 = "\\\""
  L2_2 = L2_2(L3_2, L4_2, L5_2)
  L3_2 = "\""
  L1_2 = L1_2 .. L2_2 .. L3_2
  return L1_2
end

L4_1 = {}
L4_1["\a"] = "\\a"
L4_1["\b"] = "\\b"
L4_1["\f"] = "\\f"
L4_1["\n"] = "\\n"
L4_1["\r"] = "\\r"
L4_1["\t"] = "\\t"
L4_1["\v"] = "\\v"
L5_1 = {}
L6_1 = 0
L7_1 = 31
L8_1 = 1
for L9_1 = L6_1, L7_1, L8_1 do
  L10_1 = string
  L10_1 = L10_1.char
  L11_1 = L9_1
  L10_1 = L10_1(L11_1)
  L11_1 = L4_1[L10_1]
  if not L11_1 then
    L11_1 = "\\"
    L12_1 = L9_1
    L11_1 = L11_1 .. L12_1
    L4_1[L10_1] = L11_1
    L11_1 = string
    L11_1 = L11_1.format
    L12_1 = "\\%03d"
    L13_1 = L9_1
    L11_1 = L11_1(L12_1, L13_1)
    L5_1[L10_1] = L11_1
  end
end

function L6_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L2_2 = A0_2
  L1_2 = A0_2.gsub
  L3_2 = "\\"
  L4_2 = "\\\\"
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  L2_2 = L1_2
  L1_2 = L1_2.gsub
  L3_2 = "(%c)%f[0-9]"
  L4_2 = L5_1
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  L2_2 = L1_2
  L1_2 = L1_2.gsub
  L3_2 = "%c"
  L4_2 = L4_1
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  return L1_2
end

function L7_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = type
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = A0_2
  L1_2 = A0_2.match
  L3_2 = "^[_%a][_%a%d]*$"
  L1_2 = L1_2 == "string" and L1_2
  return L1_2
end

function L8_1(A0_2, A1_2)
  local L2_2, L3_2
  L2_2 = type
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L2_2 = L2_2 == "number" and 1 <= A0_2 and A0_2 <= A1_2
  return L2_2
end

L9_1 = {}
L9_1.number = 1
L9_1.boolean = 2
L9_1.string = 3
L9_1.table = 4
L9_1["function"] = 5
L9_1.userdata = 6
L9_1.thread = 7

function L10_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = type
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = type
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  if L2_2 == L3_2 and (L2_2 == "string" or L2_2 == "number") then
    L4_2 = A0_2 < A1_2
    return L4_2
  end
  L4_2 = L9_1
  L4_2 = L4_2[L2_2]
  L5_2 = L9_1
  L5_2 = L5_2[L3_2]
  if L4_2 and L5_2 then
    L6_2 = L9_1
    L6_2 = L6_2[L2_2]
    L7_2 = L9_1
    L7_2 = L7_2[L3_2]
    L6_2 = L6_2 < L7_2
    return L6_2
  elseif L4_2 then
    L6_2 = true
    return L6_2
  elseif L5_2 then
    L6_2 = false
    return L6_2
  end
  L6_2 = L2_2 < L3_2
  return L6_2
end

function L11_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = 1
  L2_2 = rawget
  L3_2 = A0_2
  L4_2 = L1_2
  L2_2 = L2_2(L3_2, L4_2)
  while L2_2 ~= nil do
    L1_2 = L1_2 + 1
    L3_2 = rawget
    L4_2 = A0_2
    L5_2 = L1_2
    L3_2 = L3_2(L4_2, L5_2)
    L2_2 = L3_2
  end
  L3_2 = L1_2 - 1
  return L3_2
end

function L12_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = {}
  L2_2 = 0
  L3_2 = L11_1
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  L4_2 = L2_1
  L5_2 = A0_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    L9_2 = L8_1
    L10_2 = L7_2
    L11_2 = L3_2
    L9_2 = L9_2(L10_2, L11_2)
    if not L9_2 then
      L2_2 = L2_2 + 1
      L1_2[L2_2] = L7_2
    end
  end
  L4_2 = table
  L4_2 = L4_2.sort
  L5_2 = L1_2
  L6_2 = L10_1
  L4_2(L5_2, L6_2)
  L4_2 = L1_2
  L5_2 = L2_2
  L6_2 = L3_2
  return L4_2, L5_2, L6_2
end

function L13_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  if not A1_2 then
    L2_2 = {}
    A1_2 = L2_2
  end
  L2_2 = type
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "table" then
    L2_2 = A1_2[A0_2]
    if not L2_2 then
      A1_2[A0_2] = 1
      L2_2 = L2_1
      L3_2 = A0_2
      L2_2, L3_2, L4_2 = L2_2(L3_2)
      for L5_2, L6_2 in L2_2, L3_2, L4_2 do
        L7_2 = L13_1
        L8_2 = L5_2
        L9_2 = A1_2
        L7_2(L8_2, L9_2)
        L7_2 = L13_1
        L8_2 = L6_2
        L9_2 = A1_2
        L7_2(L8_2, L9_2)
      end
      L2_2 = L13_1
      L3_2 = getmetatable
      L4_2 = A0_2
      L3_2 = L3_2(L4_2)
      L4_2 = A1_2
      L2_2(L3_2, L4_2)
    else
      L2_2 = A1_2[A0_2]
      L2_2 = L2_2 + 1
      A1_2[A0_2] = L2_2
    end
  end
  return A1_2
end

function L14_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = {}
  L2_2 = #A0_2
  L3_2 = 1
  L4_2 = L2_2
  L5_2 = 1
  for L6_2 = L3_2, L4_2, L5_2 do
    L7_2 = A0_2[L6_2]
    L1_2[L6_2] = L7_2
  end
  L3_2 = L1_2
  L4_2 = L2_2
  return L3_2, L4_2
end

function L15_1(A0_2, ...)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = {}
  L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2 = ...
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  L2_2[4] = L6_2
  L2_2[5] = L7_2
  L2_2[6] = L8_2
  L2_2[7] = L9_2
  L2_2[8] = L10_2
  L3_2 = L14_1
  L4_2 = A0_2
  L3_2, L4_2 = L3_2(L4_2)
  L5_2 = 1
  L6_2 = #L2_2
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L9_2 = L4_2 + L8_2
    L10_2 = L2_2[L8_2]
    L3_2[L9_2] = L10_2
  end
  return L3_2
end

function L16_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2
  if A1_2 == nil then
    L4_2 = nil
    return L4_2
  end
  L4_2 = A3_2[A1_2]
  if L4_2 then
    L4_2 = A3_2[A1_2]
    return L4_2
  end
  L4_2 = A0_2
  L5_2 = A1_2
  L6_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = type
  L6_2 = L4_2
  L5_2 = L5_2(L6_2)
  if L5_2 == "table" then
    L5_2 = {}
    A3_2[A1_2] = L5_2
    L6_2 = nil
    L7_2 = L2_1
    L8_2 = L4_2
    L7_2, L8_2, L9_2 = L7_2(L8_2)
    for L10_2, L11_2 in L7_2, L8_2, L9_2 do
      L12_2 = L16_1
      L13_2 = A0_2
      L14_2 = L10_2
      L15_2 = L15_1
      L16_2 = A2_2
      L17_2 = L10_2
      L18_2 = L0_1
      L18_2 = L18_2.KEY
      L15_2 = L15_2(L16_2, L17_2, L18_2)
      L16_2 = A3_2
      L12_2 = L12_2(L13_2, L14_2, L15_2, L16_2)
      L6_2 = L12_2
      if L6_2 ~= nil then
        L12_2 = L16_1
        L13_2 = A0_2
        L14_2 = L11_2
        L15_2 = L15_1
        L16_2 = A2_2
        L17_2 = L6_2
        L15_2 = L15_2(L16_2, L17_2)
        L16_2 = A3_2
        L12_2 = L12_2(L13_2, L14_2, L15_2, L16_2)
        L5_2[L6_2] = L12_2
      end
    end
    L7_2 = L16_1
    L8_2 = A0_2
    L9_2 = getmetatable
    L10_2 = L4_2
    L9_2 = L9_2(L10_2)
    L10_2 = L15_1
    L11_2 = A2_2
    L12_2 = L0_1
    L12_2 = L12_2.METATABLE
    L10_2 = L10_2(L11_2, L12_2)
    L11_2 = A3_2
    L7_2 = L7_2(L8_2, L9_2, L10_2, L11_2)
    L8_2 = type
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    if L8_2 ~= "table" then
      L7_2 = nil
    end
    L8_2 = setmetatable
    L9_2 = L5_2
    L10_2 = L7_2
    L8_2(L9_2, L10_2)
    L4_2 = L5_2
  end
  return L4_2
end

L17_1 = {}
L18_1 = {}
L18_1.__index = L17_1

function L19_1(A0_2, ...)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L2_2 = {}
  L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2 = ...
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  L2_2[4] = L6_2
  L2_2[5] = L7_2
  L2_2[6] = L8_2
  L2_2[7] = L9_2
  L3_2 = A0_2.buffer
  L4_2 = #L3_2
  L5_2 = 1
  L6_2 = #L2_2
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L4_2 = L4_2 + 1
    L9_2 = L2_2[L8_2]
    L3_2[L4_2] = L9_2
  end
end

L17_1.puts = L19_1

function L19_1(A0_2, A1_2)
  local L2_2
  L2_2 = A0_2.level
  L2_2 = L2_2 + 1
  A0_2.level = L2_2
  L2_2 = A1_2
  L2_2()
  L2_2 = A0_2.level
  L2_2 = L2_2 - 1
  A0_2.level = L2_2
end

L17_1.down = L19_1

function L19_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = A0_2
  L1_2 = A0_2.puts
  L3_2 = A0_2.newline
  L4_2 = string
  L4_2 = L4_2.rep
  L5_2 = A0_2.indent
  L6_2 = A0_2.level
  L4_2, L5_2, L6_2 = L4_2(L5_2, L6_2)
  L1_2(L2_2, L3_2, L4_2, L5_2, L6_2)
end

L17_1.tabify = L19_1

function L19_1(A0_2, A1_2)
  local L2_2
  L2_2 = A0_2.ids
  L2_2 = L2_2[A1_2]
  L2_2 = L2_2 ~= nil
  return L2_2
end

L17_1.alreadyVisited = L19_1

function L19_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = A0_2.ids
  L2_2 = L2_2[A1_2]
  if not L2_2 then
    L3_2 = type
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
    L4_2 = A0_2.maxIds
    L4_2 = L4_2[L3_2]
    if not L4_2 then
      L4_2 = 0
    end
    L2_2 = L4_2 + 1
    L4_2 = A0_2.maxIds
    L4_2[L3_2] = L2_2
    L4_2 = A0_2.ids
    L4_2[A1_2] = L2_2
  end
  L3_2 = L1_1
  L4_2 = L2_2
  return L3_2(L4_2)
end

L17_1.getId = L19_1

function L19_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L7_1
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 then
    L3_2 = A0_2
    L2_2 = A0_2.puts
    L4_2 = A1_2
    return L2_2(L3_2, L4_2)
  end
  L3_2 = A0_2
  L2_2 = A0_2.puts
  L4_2 = "["
  L2_2(L3_2, L4_2)
  L3_2 = A0_2
  L2_2 = A0_2.putValue
  L4_2 = A1_2
  L2_2(L3_2, L4_2)
  L3_2 = A0_2
  L2_2 = A0_2.puts
  L4_2 = "]"
  L2_2(L3_2, L4_2)
end

L17_1.putKey = L19_1

function L19_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = L0_1
  L2_2 = L2_2.KEY
  if A1_2 ~= L2_2 then
    L2_2 = L0_1
    L2_2 = L2_2.METATABLE
    if A1_2 ~= L2_2 then
      goto lbl_15
    end
  end
  L3_2 = A0_2
  L2_2 = A0_2.puts
  L4_2 = L1_1
  L5_2 = A1_2
  L4_2, L5_2, L6_2, L7_2, L8_2 = L4_2(L5_2)
  L2_2(L3_2, L4_2, L5_2, L6_2, L7_2, L8_2)
  goto lbl_84
  ::lbl_15::
  L3_2 = A0_2
  L2_2 = A0_2.alreadyVisited
  L4_2 = A1_2
  L2_2 = L2_2(L3_2, L4_2)
  if L2_2 then
    L3_2 = A0_2
    L2_2 = A0_2.puts
    L4_2 = "<table "
    L6_2 = A0_2
    L5_2 = A0_2.getId
    L7_2 = A1_2
    L5_2 = L5_2(L6_2, L7_2)
    L6_2 = ">"
    L2_2(L3_2, L4_2, L5_2, L6_2)
  else
    L2_2 = A0_2.level
    L3_2 = A0_2.depth
    if L2_2 >= L3_2 then
      L3_2 = A0_2
      L2_2 = A0_2.puts
      L4_2 = "{...}"
      L2_2(L3_2, L4_2)
    else
      L2_2 = A0_2.tableAppearances
      L2_2 = L2_2[A1_2]
      if 1 < L2_2 then
        L3_2 = A0_2
        L2_2 = A0_2.puts
        L4_2 = "<"
        L6_2 = A0_2
        L5_2 = A0_2.getId
        L7_2 = A1_2
        L5_2 = L5_2(L6_2, L7_2)
        L6_2 = ">"
        L2_2(L3_2, L4_2, L5_2, L6_2)
      end
      L2_2 = L12_1
      L3_2 = A1_2
      L2_2, L3_2, L4_2 = L2_2(L3_2)
      L5_2 = getmetatable
      L6_2 = A1_2
      L5_2 = L5_2(L6_2)
      L7_2 = A0_2
      L6_2 = A0_2.puts
      L8_2 = "{"
      L6_2(L7_2, L8_2)
      L7_2 = A0_2
      L6_2 = A0_2.down
      
      function L8_2()
        local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
        L0_3 = 0
        L1_3 = 1
        L2_3 = L4_2
        L3_3 = 1
        for L4_3 = L1_3, L2_3, L3_3 do
          if 0 < L0_3 then
            L5_3 = A0_2
            L6_3 = L5_3
            L5_3 = L5_3.puts
            L7_3 = ","
            L5_3(L6_3, L7_3)
          end
          L5_3 = A0_2
          L6_3 = L5_3
          L5_3 = L5_3.puts
          L7_3 = " "
          L5_3(L6_3, L7_3)
          L5_3 = A0_2
          L6_3 = L5_3
          L5_3 = L5_3.putValue
          L7_3 = A1_2
          L7_3 = L7_3[L4_3]
          L5_3(L6_3, L7_3)
          L0_3 = L0_3 + 1
        end
        L1_3 = 1
        L2_3 = L3_2
        L3_3 = 1
        for L4_3 = L1_3, L2_3, L3_3 do
          L5_3 = L2_2
          L5_3 = L5_3[L4_3]
          if 0 < L0_3 then
            L6_3 = A0_2
            L7_3 = L6_3
            L6_3 = L6_3.puts
            L8_3 = ","
            L6_3(L7_3, L8_3)
          end
          L6_3 = A0_2
          L7_3 = L6_3
          L6_3 = L6_3.tabify
          L6_3(L7_3)
          L6_3 = A0_2
          L7_3 = L6_3
          L6_3 = L6_3.putKey
          L8_3 = L5_3
          L6_3(L7_3, L8_3)
          L6_3 = A0_2
          L7_3 = L6_3
          L6_3 = L6_3.puts
          L8_3 = " = "
          L6_3(L7_3, L8_3)
          L6_3 = A0_2
          L7_3 = L6_3
          L6_3 = L6_3.putValue
          L8_3 = A1_2
          L8_3 = L8_3[L5_3]
          L6_3(L7_3, L8_3)
          L0_3 = L0_3 + 1
        end
        L1_3 = type
        L2_3 = L5_2
        L1_3 = L1_3(L2_3)
        if L1_3 == "table" then
          if 0 < L0_3 then
            L1_3 = A0_2
            L2_3 = L1_3
            L1_3 = L1_3.puts
            L3_3 = ","
            L1_3(L2_3, L3_3)
          end
          L1_3 = A0_2
          L2_3 = L1_3
          L1_3 = L1_3.tabify
          L1_3(L2_3)
          L1_3 = A0_2
          L2_3 = L1_3
          L1_3 = L1_3.puts
          L3_3 = "<metatable> = "
          L1_3(L2_3, L3_3)
          L1_3 = A0_2
          L2_3 = L1_3
          L1_3 = L1_3.putValue
          L3_3 = L5_2
          L1_3(L2_3, L3_3)
        end
      end
      
      L6_2(L7_2, L8_2)
      if not (0 < L3_2) then
        L6_2 = type
        L7_2 = L5_2
        L6_2 = L6_2(L7_2)
        if L6_2 ~= "table" then
          goto lbl_75
        end
      end
      L7_2 = A0_2
      L6_2 = A0_2.tabify
      L6_2(L7_2)
      goto lbl_80
      ::lbl_75::
      if 0 < L4_2 then
        L7_2 = A0_2
        L6_2 = A0_2.puts
        L8_2 = " "
        L6_2(L7_2, L8_2)
      end
      ::lbl_80::
      L7_2 = A0_2
      L6_2 = A0_2.puts
      L8_2 = "}"
      L6_2(L7_2, L8_2)
    end
  end
  ::lbl_84::
end

L17_1.putTable = L19_1

function L19_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = type
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  if L2_2 == "string" then
    L4_2 = A0_2
    L3_2 = A0_2.puts
    L5_2 = L3_1
    L6_2 = L6_1
    L7_2 = A1_2
    L6_2, L7_2, L8_2, L9_2, L10_2 = L6_2(L7_2)
    L5_2, L6_2, L7_2, L8_2, L9_2, L10_2 = L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
    L3_2(L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2)
  elseif L2_2 == "number" or L2_2 == "boolean" or L2_2 == "nil" or L2_2 == "cdata" or L2_2 == "ctype" then
    L4_2 = A0_2
    L3_2 = A0_2.puts
    L5_2 = L1_1
    L6_2 = A1_2
    L5_2, L6_2, L7_2, L8_2, L9_2, L10_2 = L5_2(L6_2)
    L3_2(L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2)
  elseif L2_2 == "table" then
    L4_2 = A0_2
    L3_2 = A0_2.putTable
    L5_2 = A1_2
    L3_2(L4_2, L5_2)
  else
    L4_2 = A0_2
    L3_2 = A0_2.puts
    L5_2 = "<"
    L6_2 = L2_2
    L7_2 = " "
    L9_2 = A0_2
    L8_2 = A0_2.getId
    L10_2 = A1_2
    L8_2 = L8_2(L9_2, L10_2)
    L9_2 = ">"
    L3_2(L4_2, L5_2, L6_2, L7_2, L8_2, L9_2)
  end
end

L17_1.putValue = L19_1

function L19_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  if not A1_2 then
    L2_2 = {}
    A1_2 = L2_2
  end
  L2_2 = A1_2.depth
  if not L2_2 then
    L2_2 = math
    L2_2 = L2_2.huge
  end
  L3_2 = A1_2.newline
  if not L3_2 then
    L3_2 = "\n"
  end
  L4_2 = A1_2.indent
  if not L4_2 then
    L4_2 = "  "
  end
  L5_2 = A1_2.process
  if L5_2 then
    L6_2 = L16_1
    L7_2 = L5_2
    L8_2 = A0_2
    L9_2 = {}
    L10_2 = {}
    L6_2 = L6_2(L7_2, L8_2, L9_2, L10_2)
    A0_2 = L6_2
  end
  L6_2 = setmetatable
  L7_2 = {}
  L7_2.depth = L2_2
  L7_2.level = 0
  L8_2 = {}
  L7_2.buffer = L8_2
  L8_2 = {}
  L7_2.ids = L8_2
  L8_2 = {}
  L7_2.maxIds = L8_2
  L7_2.newline = L3_2
  L7_2.indent = L4_2
  L8_2 = L13_1
  L9_2 = A0_2
  L8_2 = L8_2(L9_2)
  L7_2.tableAppearances = L8_2
  L8_2 = L18_1
  L6_2 = L6_2(L7_2, L8_2)
  L8_2 = L6_2
  L7_2 = L6_2.putValue
  L9_2 = A0_2
  L7_2(L8_2, L9_2)
  L7_2 = table
  L7_2 = L7_2.concat
  L8_2 = L6_2.buffer
  return L7_2(L8_2)
end

L0_1.inspect = L19_1
L19_1 = setmetatable
L20_1 = L0_1
L21_1 = {}

function L22_1(A0_2, ...)
  local L2_2, L3_2
  L2_2 = L0_1
  L2_2 = L2_2.inspect
  L3_2 = ...
  return L2_2(L3_2)
end

L21_1.__call = L22_1
L19_1(L20_1, L21_1)
return L0_1
