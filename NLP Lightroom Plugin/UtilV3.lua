local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1, L36_1, L37_1, L38_1, L39_1

L1_1 = {}
L2_1 = import
L3_1 = "LrDialogs"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrFileUtils"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrFunctionContext"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrHttp"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrPathUtils"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrTasks"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrLogger"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrUUID"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrErrors"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrColor"
L11_1 = L11_1(L12_1)
L12_1 = L6_1.child
L13_1 = L3_1.chooseUniqueFileName
L14_1 = L3_1.fileAttributes
L15_1 = math
L15_1 = L15_1.floor
L16_1 = string
L16_1 = L16_1.format
L17_1 = L9_1.generateUUID
L18_1 = L6_1.getStandardFilePath
L19_1 = L6_1.leafName
L20_1 = L7_1.pcall
L21_1 = L6_1.parent
L22_1 = import
L23_1 = "LrApplication"
L22_1 = L22_1(L23_1)
L23_1 = import
L24_1 = "LrSystemInfo"
L23_1 = L23_1(L24_1)
L24_1 = require
L25_1 = "JSON"
L24_1 = L24_1(L25_1)
L25_1 = import
L26_1 = "LrPrefs"
L25_1 = L25_1(L26_1)
L25_1 = L25_1.prefsForPlugin
L25_1 = L25_1()
L26_1 = require
L27_1 = "inspect"
L26_1 = L26_1(L27_1)
L27_1 = require
L28_1 = "hashV3"
L27_1 = L27_1(L28_1)
L28_1 = L22_1.macAddressHash
L28_1 = L28_1()
L29_1 = "previews"
L30_1 = L6_1.child
L31_1 = _PLUGIN
L31_1 = L31_1.path
L32_1 = L29_1
L30_1 = L30_1(L31_1, L32_1)
L31_1 = L8_1
L32_1 = "NLPViewController"
L31_1 = L31_1(L32_1)
L32_1 = _G
L32_1 = L32_1.dev
if L32_1 == true then
  L33_1 = L31_1
  L32_1 = L31_1.enable
  L34_1 = "logfile"
  L32_1(L33_1, L34_1)
else
  L33_1 = L31_1
  L32_1 = L31_1.disable
  L32_1(L33_1)
end

function L32_1()
  local L0_2, L1_2
  L0_2 = L9_1
  L0_2 = L0_2.generateUUID
  L0_2 = L0_2()
  return L0_2
end

L1_1.generateUUID = L32_1
L32_1 = WIN_ENV
if L32_1 then
  L32_1 = " & "
  if L32_1 then
    goto lbl_96
  end
end
L32_1 = " ; "
::lbl_96::
L1_1.commandSeparator = L32_1

function L32_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L4_1
  L3_2 = L3_2.callWithContext
  L4_2 = ""
  
  function L5_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3
    L4_3 = A0_3
    L3_3 = A0_3.addCleanupHandler
    
    function L5_3()
      local L0_4, L1_4
      L0_4 = L1_3
      if L0_4 then
        L0_4 = L3_1
        L0_4 = L0_4.delete
        L1_4 = L1_3
        L0_4(L1_4)
      end
      L0_4 = L2_3
      if L0_4 then
        L0_4 = L3_1
        L0_4 = L0_4.delete
        L1_4 = L2_3
        L0_4(L1_4)
      end
    end
    
    L3_3(L4_3, L5_3)
    L3_3 = L9_1
    L3_3 = L3_3.generateUUID
    L3_3 = L3_3()
    L4_3 = A1_2
    if L4_3 == "error_only" then
      L4_3 = L12_1
      L5_3 = L18_1
      L6_3 = "temp"
      L5_3 = L5_3(L6_3)
      L6_3 = L3_3
      L7_3 = ".err"
      L6_3 = L6_3 .. L7_3
      L4_3 = L4_3(L5_3, L6_3)
      L2_3 = L4_3
      L4_3 = A0_2
      L5_3 = " 2>\""
      L6_3 = L2_3
      L7_3 = "\""
      L4_3 = L4_3 .. L5_3 .. L6_3 .. L7_3
      A0_2 = L4_3
    else
      L4_3 = A1_2
      if L4_3 then
        L4_3 = L12_1
        L5_3 = L18_1
        L6_3 = "temp"
        L5_3 = L5_3(L6_3)
        L6_3 = L3_3
        L7_3 = ".out"
        L6_3 = L6_3 .. L7_3
        L4_3 = L4_3(L5_3, L6_3)
        L1_3 = L4_3
        L4_3 = A0_2
        L5_3 = " > \""
        L6_3 = L1_3
        L7_3 = "\""
        L4_3 = L4_3 .. L5_3 .. L6_3 .. L7_3
        A0_2 = L4_3
        L4_3 = A1_2
        if L4_3 == "separate" then
          L4_3 = L12_1
          L5_3 = L18_1
          L6_3 = "temp"
          L5_3 = L5_3(L6_3)
          L6_3 = L3_3
          L7_3 = ".err"
          L6_3 = L6_3 .. L7_3
          L4_3 = L4_3(L5_3, L6_3)
          L2_3 = L4_3
          L4_3 = A0_2
          L5_3 = " 2>\""
          L6_3 = L2_3
          L7_3 = "\""
          L4_3 = L4_3 .. L5_3 .. L6_3 .. L7_3
          A0_2 = L4_3
        else
          L4_3 = A0_2
          L5_3 = " 2>&1"
          L4_3 = L4_3 .. L5_3
          A0_2 = L4_3
        end
      end
    end
    L4_3 = WIN_ENV
    if L4_3 then
      L4_3 = "\""
      L5_3 = A0_2
      L6_3 = "\""
      L4_3 = L4_3 .. L5_3 .. L6_3
      A0_2 = L4_3
    end
    L4_3 = L7_1
    L4_3 = L4_3.execute
    L5_3 = A0_2
    L4_3 = L4_3(L5_3)
    L5_3 = ""
    L6_3 = ""
    L7_3 = nil
    
    function L8_3(A0_4, A1_4)
      local L2_4, L3_4, L4_4, L5_4
      L2_4 = string
      L2_4 = L2_4.format
      L3_4 = [[
Couldn't read output:
%s
%s]]
      L4_4 = A0_4
      L5_4 = A1_4
      L2_4 = L2_4(L3_4, L4_4, L5_4)
      L3_4 = A1_2
      if L3_4 == "separate" then
        L3_4 = -1
        L4_4 = ""
        L5_4 = L2_4
        return L3_4, L4_4, L5_4
      else
        L3_4 = -1
        L4_4 = L2_4
        L5_4 = ""
        return L3_4, L4_4, L5_4
      end
    end
    
    if L1_3 then
      L9_3 = L20_1
      L10_3 = L3_1
      L10_3 = L10_3.readFile
      L11_3 = L1_3
      L9_3, L10_3 = L9_3(L10_3, L11_3)
      L5_3 = L10_3
      L7_3 = L9_3
      if not L7_3 then
        L9_3 = L8_3
        L10_3 = L1_3
        L11_3 = L5_3
        return L9_3(L10_3, L11_3)
      end
    end
    if L2_3 then
      L9_3 = L20_1
      L10_3 = L3_1
      L10_3 = L10_3.readFile
      L11_3 = L2_3
      L9_3, L10_3 = L9_3(L10_3, L11_3)
      L6_3 = L10_3
      L7_3 = L9_3
      if not L7_3 then
        L9_3 = L8_3
        L10_3 = L2_3
        L11_3 = L6_3
        return L9_3(L10_3, L11_3)
      end
    end
    L9_3 = L4_3
    L10_3 = L5_3
    L11_3 = L6_3
    return L9_3, L10_3, L11_3
  end
  
  return L3_2(L4_2, L5_2)
end

L1_1.safeExecute = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = MAC_ENV
  if L1_2 then
    L1_2 = L16_1
    L2_2 = "cd \"%s\"; "
    L3_2 = A0_2
    return L1_2(L2_2, L3_2)
  end
  L2_2 = A0_2
  L1_2 = A0_2.sub
  L3_2 = 1
  L4_2 = 2
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  if L1_2 == "\\\\" then
    L1_2 = error
    L2_2 = "Util.cdCommand: UNC paths not allowed: "
    L3_2 = A0_2
    L2_2 = L2_2 .. L3_2
    L1_2(L2_2)
  end
  L1_2 = A0_2
  L2_2 = ""
  while true do
    L3_2 = L21_1
    L4_2 = L1_2
    L3_2 = L3_2(L4_2)
    if L3_2 == nil then
      break
    end
    L4_2 = "\\"
    L5_2 = L19_1
    L6_2 = L1_2
    L5_2 = L5_2(L6_2)
    L6_2 = L2_2
    L2_2 = L4_2 .. L5_2 .. L6_2
    L1_2 = L3_2
  end
  L4_2 = L1_2
  L3_2 = L1_2.sub
  L5_2 = -1
  L3_2 = L3_2(L4_2, L5_2)
  if L3_2 == "\\" then
    L4_2 = L1_2
    L3_2 = L1_2.sub
    L5_2 = 1
    L6_2 = -2
    L3_2 = L3_2(L4_2, L5_2, L6_2)
    L1_2 = L3_2
  end
  if L2_2 == "" then
    L2_2 = "\\"
  end
  L3_2 = L16_1
  L4_2 = "%s & cd \"%s\" & "
  L5_2 = L1_2
  L6_2 = L2_2
  return L3_2(L4_2, L5_2, L6_2)
end

L1_1.cdCommand = L32_1

function L32_1(A0_2)
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

L1_1.quote4Win = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = "\""
  L2_2 = A0_2
  L3_2 = "\""
  L1_2 = L1_2 .. L2_2 .. L3_2
  return L1_2
end

L1_1.quote = L32_1

function L32_1(A0_2)
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

L1_1.round = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = {}
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L2_2[L6_2] = L7_2
  end
  L3_2 = pairs
  L4_2 = A1_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L2_2[L6_2] = L7_2
  end
  return L2_2
end

L1_1.union = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = A0_2[1]
  L2_2 = 1
  L3_2 = #A0_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = A0_2[L5_2]
    if L1_2 < L6_2 then
      L1_2 = A0_2[L5_2]
    end
  end
  return L1_2
end

L1_1.max = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = A0_2[1]
  L2_2 = 1
  L3_2 = #A0_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = A0_2[L5_2]
    if L1_2 > L6_2 then
      L1_2 = A0_2[L5_2]
    end
  end
  return L1_2
end

L1_1.min = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = {}
  L2_2 = pairs
  L3_2 = A0_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    L7_2 = L1_2[L6_2]
    if L7_2 == nil then
      L1_2[L6_2] = 1
    else
      L7_2 = L1_2[L6_2]
      L7_2 = L7_2 + 1
      L1_2[L6_2] = L7_2
    end
  end
  L2_2 = 0
  L3_2 = pairs
  L4_2 = L1_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    if L7_2 > L2_2 then
      L2_2 = L7_2
    end
  end
  L3_2 = {}
  L4_2 = pairs
  L5_2 = L1_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    if L8_2 == L2_2 then
      L9_2 = table
      L9_2 = L9_2.insert
      L10_2 = L3_2
      L11_2 = L7_2
      L9_2(L10_2, L11_2)
    end
  end
  return L3_2
end

L1_1.mode = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L1_2 = 0
  L2_2 = 0
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L8_2 = type
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    if L8_2 == "number" then
      L1_2 = L1_2 + L7_2
      L2_2 = L2_2 + 1
    end
  end
  L3_2 = L1_2 / L2_2
  return L3_2
end

L1_1.mean = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = 2.2
  L2_2 = 0
  L3_2 = 0
  L4_2 = pairs
  L5_2 = A0_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    L9_2 = type
    L10_2 = L8_2
    L9_2 = L9_2(L10_2)
    if L9_2 == "number" then
      L9_2 = L8_2 ^ L1_2
      L2_2 = L2_2 + L9_2
      L3_2 = L3_2 + 1
    end
  end
  L4_2 = L2_2 / L3_2
  L5_2 = 1 / L1_2
  L4_2 = L4_2 ^ L5_2
  return L4_2
end

L1_1.rgbAvg = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L1_2 = 0
  L2_2 = 0
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L8_2 = type
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    if L8_2 == "number" then
      L8_2 = L7_2 * 256
      L8_2 = L8_2 ^ 2
      L1_2 = L1_2 + L8_2
      L2_2 = L2_2 + 1
    end
  end
  L3_2 = math
  L3_2 = L3_2.sqrt
  L4_2 = L1_2 / L2_2
  L3_2 = L3_2(L4_2)
  L3_2 = L3_2 / 256
  return L3_2
end

L1_1.rgbAvgInPercent = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L3_2 = 0
  L4_2 = 0
  L5_2 = nil
  L6_2 = L1_1
  L6_2 = L6_2.mean
  L7_2 = A0_2
  L6_2 = L6_2(L7_2)
  L1_2 = L6_2
  L6_2 = pairs
  L7_2 = A0_2
  L6_2, L7_2, L8_2 = L6_2(L7_2)
  for L9_2, L10_2 in L6_2, L7_2, L8_2 do
    L11_2 = type
    L12_2 = L10_2
    L11_2 = L11_2(L12_2)
    if L11_2 == "number" then
      L2_2 = L10_2 - L1_2
      L11_2 = L2_2 * L2_2
      L3_2 = L3_2 + L11_2
      L4_2 = L4_2 + 1
    end
  end
  L6_2 = math
  L6_2 = L6_2.sqrt
  L7_2 = L4_2 - 1
  L7_2 = L3_2 / L7_2
  L6_2 = L6_2(L7_2)
  L5_2 = L6_2
  L6_2 = L5_2
  L7_2 = L1_2
  return L6_2, L7_2
end

L1_1.standardDeviation = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = math
  L1_2 = L1_2.huge
  L1_2 = -L1_2
  L2_2 = math
  L2_2 = L2_2.huge
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L8_2 = type
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    if L8_2 == "number" then
      L8_2 = math
      L8_2 = L8_2.max
      L9_2 = L1_2
      L10_2 = L7_2
      L8_2 = L8_2(L9_2, L10_2)
      L1_2 = L8_2
      L8_2 = math
      L8_2 = L8_2.min
      L9_2 = L2_2
      L10_2 = L7_2
      L8_2 = L8_2(L9_2, L10_2)
      L2_2 = L8_2
    end
  end
  L3_2 = L1_2
  L4_2 = L2_2
  return L3_2, L4_2
end

L1_1.maxmin = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = A0_2 / 255
  L3_2 = A1_2 / 2.2
  L3_2 = 2 ^ L3_2
  L3_2 = L2_2 * L3_2
  if 1 < L3_2 then
    L3_2 = 1
  end
  L4_2 = L3_2 * 255
  return L4_2
end

L1_1.adjustExposure = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L1_2 = {}
  L3_2 = A0_2
  L2_2 = A0_2.gmatch
  L4_2 = [[
[^
]+]]
  L2_2, L3_2, L4_2 = L2_2(L3_2, L4_2)
  for L5_2 in L2_2, L3_2, L4_2 do
    L6_2 = table
    L6_2 = L6_2.insert
    L7_2 = L1_2
    L8_2 = L5_2
    L6_2(L7_2, L8_2)
  end
  return L1_2
end

L1_1.lines = L32_1

function L32_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L3_2 = A0_2 / 255
  L4_2 = A1_2 / 255
  A2_2 = A2_2 / 255
  A1_2 = L4_2
  A0_2 = L3_2
  L3_2 = math
  L3_2 = L3_2.max
  L4_2 = A0_2
  L5_2 = A1_2
  L6_2 = A2_2
  L3_2 = L3_2(L4_2, L5_2, L6_2)
  L4_2 = math
  L4_2 = L4_2.min
  L5_2 = A0_2
  L6_2 = A1_2
  L7_2 = A2_2
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  L5_2 = nil
  L6_2 = nil
  L7_2 = nil
  L7_2 = L3_2
  L8_2 = L3_2 - L4_2
  if L3_2 == 0 then
    L6_2 = 0
  else
    L6_2 = L8_2 / L3_2
  end
  if L3_2 == L4_2 then
    L5_2 = 0
  else
    if L3_2 == A0_2 then
      L9_2 = A1_2 - A2_2
      L5_2 = L9_2 / L8_2
      if A2_2 > A1_2 then
        L5_2 = L5_2 + 6
      end
    elseif L3_2 == A1_2 then
      L9_2 = A2_2 - A0_2
      L9_2 = L9_2 / L8_2
      L5_2 = L9_2 + 2
    elseif L3_2 == A2_2 then
      L9_2 = A0_2 - A1_2
      L9_2 = L9_2 / L8_2
      L5_2 = L9_2 + 4
    end
    L5_2 = L5_2 / 6
  end
  L9_2 = L5_2 * 360
  L10_2 = L6_2
  L11_2 = L7_2
  return L9_2, L10_2, L11_2
end

L1_1.rgbToHsv = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "([^0-9a-zA-Z !'()*._~-])"
  
  function L4_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3
    L1_3 = string
    L1_3 = L1_3.format
    L2_3 = "%%%02X"
    L3_3 = string
    L3_3 = L3_3.byte
    L4_3 = A0_3
    L3_3, L4_3 = L3_3(L4_3)
    return L1_3(L2_3, L3_3, L4_3)
  end
  
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = " "
  L4_2 = "+"
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  return A0_2
end

escape = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = "https://api.lemonsqueezy.com/v1/licenses/activate"
  L2_2 = L1_1
  L2_2 = L2_2.v3
  L2_2 = L2_2()
  L3_2 = escape
  L4_2 = L2_2
  L3_2 = L3_2(L4_2)
  L4_2 = "license_key="
  L5_2 = A0_2
  L6_2 = "&instance_name="
  L7_2 = L3_2
  L4_2 = L4_2 .. L5_2 .. L6_2 .. L7_2
  L5_2 = {}
  L6_2 = auth_header
  L7_2 = {}
  L7_2.field = "Accept"
  L7_2.value = "application/json"
  L8_2 = {}
  L8_2.field = "Content-Type"
  L8_2.value = "application/x-www-form-urlencoded"
  L9_2 = {}
  L9_2.field = "User-Agent"
  L9_2.value = "Lightroom Plugin"
  L10_2 = {}
  L10_2.field = "Cookie"
  L10_2.value = "GARBAGE"
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L5_2[4] = L9_2
  L5_2[5] = L10_2
  L6_2 = L5_1
  L6_2 = L6_2.post
  L7_2 = L1_2
  L8_2 = L4_2
  L9_2 = L5_2
  L10_2 = "POST"
  L11_2 = 10
  L6_2, L7_2 = L6_2(L7_2, L8_2, L9_2, L10_2, L11_2)
  L8_2 = L6_2
  L9_2 = L7_2
  return L8_2, L9_2
end

L1_1.activate = L32_1

function L32_1()
  local L0_2, L1_2, L2_2
  L0_2 = L27_1
  L0_2 = L0_2.blake3
  L1_2 = L22_1
  L1_2 = L1_2.macAddressHash
  L1_2 = L1_2()
  L2_2 = "kodak"
  L0_2 = L0_2(L1_2, L2_2)
  return L0_2
end

L1_1.v3 = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = "https://api.lemonsqueezy.com/v1/licenses/deactivate"
  L3_2 = "license_key="
  L4_2 = A0_2
  L5_2 = "&instance_id="
  L6_2 = A1_2
  L3_2 = L3_2 .. L4_2 .. L5_2 .. L6_2
  L4_2 = {}
  L5_2 = {}
  L5_2.field = "Accept"
  L5_2.value = "application/json"
  L6_2 = {}
  L6_2.field = "Content-Type"
  L6_2.value = "application/x-www-form-urlencoded"
  L7_2 = {}
  L7_2.field = "User-Agent"
  L7_2.value = "Lightroom Plugin"
  L8_2 = {}
  L8_2.field = "Cookie"
  L8_2.value = "GARBAGE"
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L4_2[4] = L8_2
  L5_2 = L5_1
  L5_2 = L5_2.post
  L6_2 = L2_2
  L7_2 = L3_2
  L8_2 = L4_2
  L9_2 = "POST"
  L10_2 = 10
  L5_2, L6_2 = L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
  L7_2 = L5_2
  L8_2 = L6_2
  return L7_2, L8_2
end

L1_1.deactivate = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = "https://api.lemonsqueezy.com/v1/licenses/validate"
  L3_2 = "license_key="
  L4_2 = A0_2
  L5_2 = "&instance_id="
  L6_2 = A1_2
  L3_2 = L3_2 .. L4_2 .. L5_2 .. L6_2
  L4_2 = {}
  L5_2 = auth_header
  L6_2 = {}
  L6_2.field = "Accept"
  L6_2.value = "application/json"
  L7_2 = {}
  L7_2.field = "Content-Type"
  L7_2.value = "application/x-www-form-urlencoded"
  L8_2 = {}
  L8_2.field = "User-Agent"
  L8_2.value = "Lightroom Plugin"
  L9_2 = {}
  L9_2.field = "Cookie"
  L9_2.value = "GARBAGE"
  L4_2[1] = L5_2
  L4_2[2] = L6_2
  L4_2[3] = L7_2
  L4_2[4] = L8_2
  L4_2[5] = L9_2
  L5_2 = L5_1
  L5_2 = L5_2.post
  L6_2 = L2_2
  L7_2 = L3_2
  L8_2 = L4_2
  L9_2 = "POST"
  L10_2 = 10
  L5_2, L6_2 = L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
  L7_2 = L5_2
  L8_2 = L6_2
  return L7_2, L8_2
end

L1_1.validate = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = math
  L2_2 = L2_2.log10
  L3_2 = A0_2
  L2_2 = L2_2(L3_2)
  L3_2 = math
  L3_2 = L3_2.log10
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 / L3_2
  return L2_2
end

L1_1.logb = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = math
  L2_2 = L2_2.log10
  L3_2 = A1_2
  L2_2 = L2_2(L3_2)
  L3_2 = math
  L3_2 = L3_2.log10
  L4_2 = A0_2
  L3_2 = L3_2(L4_2)
  L2_2 = L2_2 / L3_2
  return L2_2
end

L1_1.logb2 = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = A0_2.s
  L1_2 = L1_2 / 100
  L2_2 = A0_2.b
  if L2_2 ~= nil then
    L2_2 = A0_2.b
    L2_2 = L2_2 / 100
    if L2_2 then
      goto lbl_11
    end
  end
  L2_2 = 0.5
  ::lbl_11::
  L3_2 = {}
  L4_2 = A0_2.h
  L3_2.h = L4_2
  L3_2.s = L1_2
  L3_2.b = L2_2
  return L3_2
end

L1_1.formatHSV = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = A0_2.s
  L1_2 = L1_2 * 100
  L2_2 = A0_2.b
  if L2_2 ~= nil then
    L2_2 = A0_2.b
    L2_2 = L2_2 * 100
    if L2_2 then
      goto lbl_11
    end
  end
  L2_2 = 0.5
  ::lbl_11::
  L3_2 = {}
  L4_2 = A0_2.h
  L3_2.h = L4_2
  L3_2.s = L1_2
  L3_2.b = L2_2
  return L3_2
end

L1_1.unformatHSV = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = math
  L1_2 = L1_2.floor
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = A0_2 - L1_2
  L3_2 = 0.16666666666666666
  if L2_2 <= L3_2 then
    return L1_2
  else
    if L2_2 > L3_2 then
      L4_2 = L3_2 * 3
      if L2_2 <= L4_2 then
        L4_2 = L1_2 + 0.333333
        return L4_2
    end
    else
      L4_2 = L3_2 * 3
      if L2_2 > L4_2 then
        L4_2 = L3_2 * 5
        if L2_2 <= L4_2 then
          L4_2 = L1_2 + 0.666666
          return L4_2
      end
      else
        L4_2 = L3_2 * 5
        if L2_2 > L4_2 then
          L4_2 = L1_2 + 1
          return L4_2
        end
      end
    end
  end
end

L1_1.roundEV = L32_1

function L32_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2
  L3_2 = L1_1
  L3_2 = L3_2.logb2
  L4_2 = 2
  L5_2 = A1_2 ^ 2
  L5_2 = L5_2 / A0_2
  L3_2 = L3_2(L4_2, L5_2)
  L4_2 = L1_1
  L4_2 = L4_2.logb2
  L5_2 = 2
  L6_2 = A2_2 / 100
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = L3_2 + L4_2
  L6_2 = L1_1
  L6_2 = L6_2.roundEV
  L7_2 = L5_2
  L6_2 = L6_2(L7_2)
  L5_2 = L6_2
  return L5_2
end

L1_1.exposureValue = L32_1

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2
  L1_2 = {}
  L2_2 = pairs
  L3_2 = A0_2[1]
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    L7_2 = {}
    L1_2[L5_2] = L7_2
  end
  L2_2 = 1
  L3_2 = #A0_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = pairs
    L7_2 = A0_2[L5_2]
    L6_2, L7_2, L8_2 = L6_2(L7_2)
    for L9_2, L10_2 in L6_2, L7_2, L8_2 do
      L11_2 = table
      L11_2 = L11_2.insert
      L12_2 = L1_2[L9_2]
      L13_2 = L10_2
      L11_2(L12_2, L13_2)
    end
  end
  return L1_2
end

L1_1.combineTablesByKey = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2
  
  function L2_2(A0_3, A1_3)
    local L2_3, L3_3, L4_3, L5_3
    L2_3 = A1_2
    L2_3 = A0_3[L2_3]
    L3_3 = A1_2
    L3_3 = A1_3[L3_3]
    L4_3 = type
    L5_3 = L2_3
    L4_3 = L4_3(L5_3)
    if L4_3 == "string" then
      L4_3 = type
      L5_3 = L3_3
      L4_3 = L4_3(L5_3)
      if L4_3 == "string" then
        L5_3 = L2_3
        L4_3 = L2_3.lower
        L4_3 = L4_3(L5_3)
        L2_3 = L4_3
        L5_3 = L3_3
        L4_3 = L3_3.lower
        L4_3 = L4_3(L5_3)
        L3_3 = L4_3
      end
    end
    L4_3 = L2_3 < L3_3
    return L4_3
  end
  
  L3_2 = table
  L3_2 = L3_2.sort
  L4_2 = A0_2
  L5_2 = L2_2
  L3_2(L4_2, L5_2)
  return A0_2
end

L1_1.sortByKey = L32_1

function L32_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L2_2 = {}
  L3_2 = A0_2.s
  L4_2 = A0_2.b
  L5_2 = A0_2.h
  if 360 <= L5_2 then
    L5_2 = L5_2 - 360
  end
  if L3_2 == 0 then
    L6_2 = {}
    L7_2 = L4_2
    L8_2 = L4_2
    L9_2 = L4_2
    L6_2[1] = L7_2
    L6_2[2] = L8_2
    L6_2[3] = L9_2
    L2_2 = L6_2
  end
  L6_2 = math
  L6_2 = L6_2.floor
  L7_2 = L5_2 / 60
  L6_2 = L6_2(L7_2)
  L7_2 = L5_2 / 60
  L7_2 = L7_2 - L6_2
  L8_2 = 1 - L3_2
  L8_2 = L4_2 * L8_2
  L9_2 = L3_2 * L7_2
  L9_2 = 1 - L9_2
  L9_2 = L4_2 * L9_2
  L10_2 = 1 - L7_2
  L10_2 = L3_2 * L10_2
  L10_2 = 1 - L10_2
  L10_2 = L4_2 * L10_2
  if L6_2 == 0 then
    L11_2 = {}
    L12_2 = L4_2
    L13_2 = L10_2
    L14_2 = L8_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  elseif L6_2 == 1 then
    L11_2 = {}
    L12_2 = L9_2
    L13_2 = L4_2
    L14_2 = L8_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  elseif L6_2 == 2 then
    L11_2 = {}
    L12_2 = L8_2
    L13_2 = L4_2
    L14_2 = L10_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  elseif L6_2 == 3 then
    L11_2 = {}
    L12_2 = L8_2
    L13_2 = L9_2
    L14_2 = L4_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  elseif L6_2 == 4 then
    L11_2 = {}
    L12_2 = L10_2
    L13_2 = L8_2
    L14_2 = L4_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  elseif L6_2 == 5 then
    L11_2 = {}
    L12_2 = L4_2
    L13_2 = L8_2
    L14_2 = L9_2
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    L2_2 = L11_2
  end
  if A1_2 then
    L11_2 = {}
    L12_2 = L2_2[1]
    L12_2 = L12_2 * 255
    L13_2 = L2_2[2]
    L13_2 = L13_2 * 255
    L14_2 = L2_2[3]
    L14_2 = L14_2 * 255
    L11_2[1] = L12_2
    L11_2[2] = L13_2
    L11_2[3] = L14_2
    return L11_2
  else
    return L2_2
  end
end

L1_1.HSVToRGB = L32_1

function L32_1()
  local L0_2, L1_2
end

L1_1.testLog = L32_1

function L32_1(A0_2)
  local L1_2
  L1_2 = 268435456 < A0_2
  return L1_2
end

function L33_1(A0_2)
  local L1_2
  L1_2 = A0_2 < 3.725290298461914E-9
  return L1_2
end

L34_1 = math
L34_1 = L34_1.huge

function L35_1(A0_2)
  local L1_2
  L1_2 = A0_2 ~= A0_2
  return L1_2
end

function L36_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = 1 + A0_2
  if L1_2 == 1 then
    return A0_2
  end
  L2_2 = math
  L2_2 = L2_2.log
  L3_2 = L1_2
  L2_2 = L2_2(L3_2)
  L2_2 = L2_2 * A0_2
  L3_2 = L1_2 - 1
  L2_2 = L2_2 / L3_2
  return L2_2
end

function L37_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  if A0_2 < 1 then
    L1_2 = A0_2 - A0_2
    L2_2 = A0_2 - A0_2
    L1_2 = L1_2 / L2_2
    return L1_2
  end
  L1_2 = L32_1
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  if L1_2 then
    L1_2 = L35_1
    L2_2 = A0_2
    L1_2 = L1_2(L2_2)
    if L1_2 then
      L1_2 = A0_2 + A0_2
      return L1_2
    end
    L1_2 = math
    L1_2 = L1_2.log
    L2_2 = 2
    L1_2 = L1_2(L2_2)
    L2_2 = math
    L2_2 = L2_2.log
    L3_2 = A0_2
    L2_2 = L2_2(L3_2)
    L1_2 = L1_2 + L2_2
    return L1_2
  end
  L1_2 = A0_2 + 1
  if L1_2 == 1 then
    L1_2 = 0
    return L1_2
  end
  if 2 < A0_2 then
    L1_2 = A0_2 * A0_2
    L2_2 = math
    L2_2 = L2_2.log
    L3_2 = 2 * A0_2
    L4_2 = math
    L4_2 = L4_2.sqrt
    L5_2 = L1_2 - 1
    L4_2 = L4_2(L5_2)
    L4_2 = A0_2 + L4_2
    L4_2 = 1 / L4_2
    L3_2 = L3_2 - L4_2
    return L2_2(L3_2)
  end
  L1_2 = A0_2 - 1
  L2_2 = L36_1
  L3_2 = math
  L3_2 = L3_2.sqrt
  L4_2 = 2 * L1_2
  L5_2 = L1_2 * L1_2
  L4_2 = L4_2 + L5_2
  L3_2 = L3_2(L4_2)
  L3_2 = L1_2 + L3_2
  return L2_2(L3_2)
end

function L38_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = math
  L1_2 = L1_2.abs
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = L33_1
  L3_2 = L1_2
  L2_2 = L2_2(L3_2)
  if L2_2 then
    return A0_2
  end
  L2_2 = nil
  L3_2 = L32_1
  L4_2 = L1_2
  L3_2 = L3_2(L4_2)
  if L3_2 then
    L3_2 = L35_1
    L4_2 = A0_2
    L3_2 = L3_2(L4_2)
    if L3_2 then
      L3_2 = A0_2 + A0_2
      return L3_2
    end
    L3_2 = math
    L3_2 = L3_2.log
    L4_2 = 2
    L3_2 = L3_2(L4_2)
    L4_2 = math
    L4_2 = L4_2.log
    L5_2 = L1_2
    L4_2 = L4_2(L5_2)
    L2_2 = L3_2 + L4_2
  elseif 2 < L1_2 then
    L3_2 = math
    L3_2 = L3_2.log
    L4_2 = 2 * L1_2
    L5_2 = math
    L5_2 = L5_2.sqrt
    L6_2 = L1_2 * L1_2
    L6_2 = 1 + L6_2
    L5_2 = L5_2(L6_2)
    L5_2 = L1_2 + L5_2
    L5_2 = 1 / L5_2
    L4_2 = L4_2 + L5_2
    L3_2 = L3_2(L4_2)
    L2_2 = L3_2
  else
    L3_2 = L1_2 * L1_2
    L4_2 = L36_1
    L5_2 = math
    L5_2 = L5_2.sqrt
    L6_2 = 1 + L3_2
    L5_2 = L5_2(L6_2)
    L5_2 = 1 + L5_2
    L5_2 = L3_2 / L5_2
    L5_2 = L1_2 + L5_2
    L4_2 = L4_2(L5_2)
    L2_2 = L4_2
  end
  if A0_2 < 0 then
    L3_2 = -L2_2
    if L3_2 then
      goto lbl_67
    end
  end
  L3_2 = L2_2
  ::lbl_67::
  return L3_2
end

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = math
  L1_2 = L1_2.abs
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = nil
  if L1_2 < 0.5 then
    L3_2 = L33_1
    L4_2 = L1_2
    L3_2 = L3_2(L4_2)
    if L3_2 then
      return A0_2
    end
    L2_2 = 2 * L1_2
    L3_2 = L36_1
    L4_2 = L2_2 * L1_2
    L5_2 = 1 - L1_2
    L4_2 = L4_2 / L5_2
    L4_2 = L2_2 + L4_2
    L3_2 = L3_2(L4_2)
    L2_2 = 0.5 * L3_2
  elseif L1_2 < 1 then
    L3_2 = L36_1
    L4_2 = 2 * L1_2
    L5_2 = 1 - L1_2
    L4_2 = L4_2 / L5_2
    L3_2 = L3_2(L4_2)
    L2_2 = 0.5 * L3_2
  elseif 1 < L1_2 then
    L3_2 = A0_2 - A0_2
    L4_2 = A0_2 - A0_2
    L3_2 = L3_2 / L4_2
    return L3_2
  else
    L3_2 = A0_2 / 0
    return L3_2
  end
  if A0_2 < 0 then
    L3_2 = -L2_2
    if L3_2 then
      goto lbl_47
    end
  end
  L3_2 = L2_2
  ::lbl_47::
  return L3_2
end

L1_1.atanh = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2
  L2_2 = A0_2 / 255
  if L2_2 <= 0.04045 then
    L1_2 = L2_2 / 12.92
  else
    L3_2 = L2_2 + 0.055
    L3_2 = L3_2 / 1.055
    L1_2 = L3_2 ^ 2.4
  end
  L3_2 = L1_2 * 255
  return L3_2
end

L1_1.toLinear = L39_1

function L39_1(A0_2)
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

L1_1.toSRGB = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = {}
  L2_2 = pairs
  L3_2 = A0_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    L1_2[L5_2] = L6_2
  end
  return L1_2
end

L1_1.copyTable = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = type
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = nil
  if L1_2 == "table" then
    L3_2 = {}
    L2_2 = L3_2
    L3_2 = pairs
    L4_2 = A0_2
    L3_2, L4_2, L5_2 = L3_2(L4_2)
    for L6_2, L7_2 in L3_2, L4_2, L5_2 do
      L8_2 = L1_1
      L8_2 = L8_2.deepCopy
      L9_2 = L6_2
      L8_2 = L8_2(L9_2)
      L9_2 = L1_1
      L9_2 = L9_2.deepCopy
      L10_2 = L7_2
      L9_2 = L9_2(L10_2)
      L2_2[L8_2] = L9_2
    end
    L3_2 = setmetatable
    L4_2 = L2_2
    L5_2 = L1_1
    L5_2 = L5_2.deepCopy
    L6_2 = getmetatable
    L7_2 = A0_2
    L6_2, L7_2, L8_2, L9_2, L10_2 = L6_2(L7_2)
    L5_2, L6_2, L7_2, L8_2, L9_2, L10_2 = L5_2(L6_2, L7_2, L8_2, L9_2, L10_2)
    L3_2(L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2)
  else
    L2_2 = A0_2
  end
  return L2_2
end

L1_1.deepCopy = L39_1

function L39_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = pairs
  L3_2 = A0_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    if L6_2 == A1_2 then
      L7_2 = true
      return L7_2
    end
  end
  L2_2 = false
  return L2_2
end

L1_1.isInTable = L39_1

function L39_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = pairs
  L3_2 = A0_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    if L5_2 == A1_2 then
      L7_2 = true
      return L7_2
    end
  end
  L2_2 = false
  return L2_2
end

L1_1.keyIsInTable = L39_1

function L39_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = pairs
  L3_2 = A1_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    A0_2[L6_2] = nil
  end
end

L1_1.remove_keys = L39_1

function L39_1(A0_2)
  local L1_2, L2_2
  L1_2 = next
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L1_2 = L1_2 == nil
  return L1_2
end

L1_1.tableIsEmpty = L39_1

function L39_1(A0_2, A1_2)
  local L2_2, L3_2
  while true do
    L2_2 = #A0_2
    if not (A1_2 < L2_2) then
      break
    end
    L2_2 = table
    L2_2 = L2_2.remove
    L3_2 = A0_2
    L2_2(L3_2)
  end
end

L1_1.limitTableSize = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = {}
  L3_2 = A0_2
  L2_2 = A0_2.pairs
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    L1_2[L5_2] = L6_2
  end
  return L1_2
end

L1_1.copyObservableTable = L39_1

function L39_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2
  L4_2 = A3_2 - A1_2
  L5_2 = A2_2 - A0_2
  L4_2 = L4_2 / L5_2
  return L4_2
end

L1_1.calculate_slope = L39_1

function L39_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2
  L4_2 = A3_2 - A1_2
  L4_2 = A0_2 * L4_2
  L4_2 = L4_2 + A2_2
  return L4_2
end

L1_1.calculate_y = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2
  L1_2 = {}
  L3_2 = A0_2
  L2_2 = A0_2.red
  L2_2 = L2_2(L3_2)
  L4_2 = A0_2
  L3_2 = A0_2.green
  L3_2 = L3_2(L4_2)
  L5_2 = A0_2
  L4_2 = A0_2.blue
  L4_2, L5_2 = L4_2(L5_2)
  L1_2[1] = L2_2
  L1_2[2] = L3_2
  L1_2[3] = L4_2
  L1_2[4] = L5_2
  return L1_2
end

L1_1.colorToArray = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = L11_1
  L2_2 = A0_2[1]
  L3_2 = A0_2[2]
  L4_2 = A0_2[3]
  return L1_2(L2_2, L3_2, L4_2)
end

L1_1.arrayToColor = L39_1

function L39_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L3_2 = {}
  L4_2 = pairs
  L5_2 = A0_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    L3_2[L8_2] = L7_2
  end
  L4_2 = L3_2[A1_2]
  L5_2 = nil
  if A2_2 then
    L6_2 = #A0_2
    if L4_2 == L6_2 then
      L5_2 = L4_2
  end
  else
    L6_2 = L4_2 + 1
    L7_2 = #A0_2
    if L6_2 <= L7_2 then
      L6_2 = L4_2 + 1
      if L6_2 then
        goto lbl_26
        L5_2 = L6_2 or L5_2
      end
    end
    L5_2 = 1
  end
  ::lbl_26::
  L6_2 = A0_2[L5_2]
  return L6_2
end

L1_1.incrementArray = L39_1

function L39_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L3_2 = {}
  L4_2 = pairs
  L5_2 = A0_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    L3_2[L8_2] = L7_2
  end
  L4_2 = L3_2[A1_2]
  L5_2 = nil
  if A2_2 and L4_2 == 1 then
    L5_2 = 1
  else
    L6_2 = L4_2 - 1
    if 0 < L6_2 then
      L6_2 = L4_2 - 1
      if L6_2 then
        goto lbl_24
        L5_2 = L6_2 or L5_2
      end
    end
    L5_2 = #A0_2
  end
  ::lbl_24::
  L6_2 = A0_2[L5_2]
  return L6_2
end

L1_1.decrementArray = L39_1

function L39_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2
  L5_2 = A0_2 + A1_2
  if A2_2 < L5_2 then
    if A4_2 then
      L5_2 = A3_2
    else
      L5_2 = A2_2
    end
  end
  if A3_2 > L5_2 then
    if A4_2 then
      L5_2 = A2_2
    else
      L5_2 = A3_2
    end
  end
  return L5_2
end

L1_1.nextInRange = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "[^%w%.%-]"
  L4_2 = "-"
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "^%s+"
  L4_2 = ""
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "%s+$"
  L4_2 = ""
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L1_2 = string
  L1_2 = L1_2.gsub
  L2_2 = A0_2
  L3_2 = "^%.+"
  L4_2 = ""
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  L1_2 = string
  L1_2 = L1_2.sub
  L2_2 = A0_2
  L3_2 = 1
  L4_2 = 255
  L1_2 = L1_2(L2_2, L3_2, L4_2)
  A0_2 = L1_2
  return A0_2
end

L1_1.sanitize_filename = L39_1

function L39_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  if A0_2 ~= nil then
    L2_2 = A0_2
    L1_2 = A0_2.len
    L1_2 = L1_2(L2_2)
    if L1_2 ~= 0 then
      goto lbl_9
    end
  end
  L1_2 = nil
  do return L1_2 end
  ::lbl_9::
  L1_2 = type
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  if L1_2 ~= "string" then
    L1_2 = error
    L2_2 = "Expected string"
    L1_2(L2_2)
    L1_2 = nil
    return L1_2
  end
  L2_2 = A0_2
  L1_2 = A0_2.find
  L3_2 = "[^%@]+$"
  L1_2 = L1_2(L2_2, L3_2)
  L3_2 = A0_2
  L2_2 = A0_2.sub
  L4_2 = 1
  L5_2 = L1_2 - 2
  L2_2 = L2_2(L3_2, L4_2, L5_2)
  L4_2 = A0_2
  L3_2 = A0_2.sub
  L5_2 = L1_2
  L6_2 = #A0_2
  L3_2 = L3_2(L4_2, L5_2, L6_2)
  if L2_2 == nil then
    L4_2 = nil
    L5_2 = "Local name is invalid"
    return L4_2, L5_2
  end
  if L3_2 ~= nil then
    L5_2 = L3_2
    L4_2 = L3_2.find
    L6_2 = "%."
    L4_2 = L4_2(L5_2, L6_2)
    if L4_2 then
      goto lbl_45
    end
  end
  L4_2 = nil
  L5_2 = "Domain is invalid"
  do return L4_2, L5_2 end
  ::lbl_45::
  L4_2 = string
  L4_2 = L4_2.sub
  L5_2 = L3_2
  L6_2 = 1
  L7_2 = 1
  L4_2 = L4_2(L5_2, L6_2, L7_2)
  if L4_2 == "." then
    L4_2 = nil
    L5_2 = "First character in domain cannot be a dot"
    return L4_2, L5_2
  end
  L4_2 = #L2_2
  if 64 < L4_2 then
    L4_2 = nil
    L5_2 = "Local name must be less than 64 characters"
    return L4_2, L5_2
  end
  L4_2 = #L3_2
  if 253 < L4_2 then
    L4_2 = nil
    L5_2 = "Domain must be less than 253 characters"
    return L4_2, L5_2
  end
  if 65 <= L1_2 then
    L4_2 = nil
    L5_2 = "Invalid @ symbol usage"
    return L4_2, L5_2
  end
  L5_2 = L2_2
  L4_2 = L2_2.find
  L6_2 = "[\"]"
  L4_2 = L4_2(L5_2, L6_2)
  L5_2 = type
  L6_2 = L4_2
  L5_2 = L5_2(L6_2)
  if L5_2 == "number" and 1 < L4_2 then
    L5_2 = nil
    L6_2 = "Invalid usage of quotes"
    return L5_2, L6_2
  end
  L6_2 = L2_2
  L5_2 = L2_2.find
  L7_2 = "%@+"
  L5_2 = L5_2(L6_2, L7_2)
  if L5_2 and L4_2 == nil then
    L5_2 = nil
    L6_2 = "Invalid @ symbol usage in local part"
    return L5_2, L6_2
  end
  L6_2 = L3_2
  L5_2 = L3_2.find
  L7_2 = "%."
  L5_2 = L5_2(L6_2, L7_2)
  if not L5_2 then
    L5_2 = nil
    L6_2 = "No TLD found in domain"
    return L5_2, L6_2
  end
  L6_2 = L3_2
  L5_2 = L3_2.find
  L7_2 = "%.%."
  L5_2 = L5_2(L6_2, L7_2)
  if L5_2 then
    L5_2 = nil
    L6_2 = "Too many periods in domain"
    return L5_2, L6_2
  end
  L6_2 = L2_2
  L5_2 = L2_2.find
  L7_2 = "%.%."
  L5_2 = L5_2(L6_2, L7_2)
  if L5_2 then
    L5_2 = nil
    L6_2 = "Too many periods in local part"
    return L5_2, L6_2
  end
  L6_2 = A0_2
  L5_2 = A0_2.match
  L7_2 = "[%w]*[%p]*%@+[%w]*[%.]?[%w]*"
  L5_2 = L5_2(L6_2, L7_2)
  if not L5_2 then
    L5_2 = nil
    L6_2 = "Email pattern test failed"
    return L5_2, L6_2
  end
  L5_2 = true
  return L5_2
end

L1_1.validEmail = L39_1

function L39_1(A0_2, A1_2)
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

L1_1.rotateCropForOrientation = L39_1
return L1_1
