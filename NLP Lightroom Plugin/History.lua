local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1
L0_1 = import
L1_1 = "LrTasks"
L0_1 = L0_1(L1_1)
L1_1 = import
L2_1 = "LrDate"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrDialogs"
L2_1 = L2_1(L3_1)
L3_1 = require
L4_1 = "NegativePresets"
L3_1 = L3_1(L4_1)
L4_1 = require
L5_1 = "UtilV3"
L4_1 = L4_1(L5_1)
L5_1 = require
L6_1 = "inspect"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrLogger"
L6_1 = L6_1(L7_1)
L7_1 = L6_1
L8_1 = "NLPViewController"
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
L8_1 = nil
L9_1 = nil
L10_1 = nil
L11_1 = nil
L12_1 = nil
L13_1 = nil

function L14_1()
  local L0_2, L1_2
  L0_2 = false
  L8_1 = L0_2
  L0_2 = nil
  L9_1 = L0_2
  L0_2 = nil
  L10_1 = L0_2
  L0_2 = 0.5
  L11_1 = L0_2
  L0_2 = 0
  L12_1 = L0_2
  L0_2 = {}
  L13_1 = L0_2
end

L15_1 = L14_1
L15_1()
L15_1 = {}
L16_1 = "colorModel"
L17_1 = "saturation"
L18_1 = "source"
L19_1 = "convertAsRoll"
L20_1 = "previewBuffer"
L21_1 = "filmBorder"
L22_1 = "selectedPreset"
L23_1 = "film"
L15_1[1] = L16_1
L15_1[2] = L17_1
L15_1[3] = L18_1
L15_1[4] = L19_1
L15_1[5] = L20_1
L15_1[6] = L21_1
L15_1[7] = L22_1
L15_1[8] = L23_1
L16_1 = {}
L17_1 = "add"
L18_1 = "manage"
L19_1 = "delete"
L20_1 = "edit"
L21_1 = "update"
L22_1 = "default"
L23_1 = "save"
L24_1 = "reload"
L25_1 = "new"
L16_1[1] = L17_1
L16_1[2] = L18_1
L16_1[3] = L19_1
L16_1[4] = L20_1
L16_1[5] = L21_1
L16_1[6] = L22_1
L16_1[7] = L23_1
L16_1[8] = L24_1
L16_1[9] = L25_1
L17_1 = {}
L18_1 = {}
L17_1.stack = L18_1

function L18_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L4_1
  L2_2 = L2_2.isInTable
  L3_2 = L15_1
  L4_2 = A0_2
  L2_2 = L2_2(L3_2, L4_2)
  if not L2_2 then
    L2_2 = L4_1
    L2_2 = L2_2.isInTable
    L3_2 = L16_1
    L4_2 = A1_2
    L2_2 = L2_2(L3_2, L4_2)
    if not L2_2 then
      goto lbl_18
    end
  end
  L2_2 = false
  do return L2_2 end
  goto lbl_20
  ::lbl_18::
  L2_2 = true
  do return L2_2 end
  ::lbl_20::
end

function L19_1()
  local L0_2, L1_2, L2_2
  L0_2 = L4_1
  L0_2 = L0_2.tableIsEmpty
  L1_2 = L17_1
  L1_2 = L1_2.stack
  L0_2 = L0_2(L1_2)
  if L0_2 then
    L0_2 = L17_1
    L0_2 = L0_2.stack
    L1_2 = {}
    L0_2[1] = L1_2
    L0_2 = L17_1
    L0_2 = L0_2.stack
    L1_2 = _G
    L1_2 = L1_2.activePhotosMeta
    L2_2 = _G
    L2_2 = L2_2.activePhotoIndex
    L1_2 = L1_2[L2_2]
    L1_2 = L1_2.settings
    L0_2[1] = L1_2
    L0_2 = 1
    L12_1 = L0_2
  end
end

function L20_1()
  local L0_2, L1_2, L2_2
  L0_2 = L17_1
  L0_2 = L0_2.stack
  L0_2 = #L0_2
  L1_2 = L12_1
  if L0_2 > L1_2 then
    L0_2 = L4_1
    L0_2 = L0_2.limitTableSize
    L1_2 = L17_1
    L1_2 = L1_2.stack
    L2_2 = L12_1
    L0_2(L1_2, L2_2)
  end
end

function L21_1()
  local L0_2, L1_2
  L0_2 = L17_1
  L0_2 = L0_2.stack
  L0_2 = #L0_2
  if 1 < L0_2 then
    L0_2 = L12_1
    if 1 < L0_2 then
      L0_2 = uiSettings
      L0_2.undoIsVisible = true
  end
  else
    L0_2 = uiSettings
    L0_2.undoIsVisible = false
  end
  L0_2 = L17_1
  L0_2 = L0_2.stack
  L0_2 = #L0_2
  if 1 < L0_2 then
    L0_2 = L12_1
    L1_2 = L17_1
    L1_2 = L1_2.stack
    L1_2 = #L1_2
    if L0_2 < L1_2 then
      L0_2 = uiSettings
      L0_2.redoIsVisible = true
  end
  else
    L0_2 = uiSettings
    L0_2.redoIsVisible = false
  end
end

function L22_1(A0_2, A1_2, A2_2)
  if A0_2 ~= "wbTemp" then
  end
  A2_2.film = "custom"
  return A2_2
end

function L23_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L18_1
  L3_2 = A0_2
  L4_2 = A1_2
  L2_2 = L2_2(L3_2, L4_2)
  if not L2_2 then
    return
  end
  L2_2 = L19_1
  L2_2()
  L2_2 = L20_1
  L2_2()
  L2_2 = L4_1
  L2_2 = L2_2.copyTable
  L3_2 = L17_1
  L3_2 = L3_2.stack
  L4_2 = L12_1
  L3_2 = L3_2[L4_2]
  L2_2 = L2_2(L3_2)
  L2_2[A0_2] = A1_2
  L3_2 = L22_1
  L4_2 = A0_2
  L5_2 = A1_2
  L6_2 = L2_2
  L3_2 = L3_2(L4_2, L5_2, L6_2)
  L2_2 = L3_2
  L3_2 = L12_1
  L3_2 = L3_2 + 1
  L12_1 = L3_2
  L3_2 = L17_1
  L3_2 = L3_2.stack
  L4_2 = L12_1
  L5_2 = {}
  L3_2[L4_2] = L5_2
  L3_2 = L17_1
  L3_2 = L3_2.stack
  L4_2 = L12_1
  L3_2[L4_2] = L2_2
  L3_2 = L21_1
  L3_2()
end

function L24_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L20_1
  L1_2()
  L1_2 = L12_1
  L1_2 = L1_2 + 1
  L12_1 = L1_2
  L1_2 = L17_1
  L1_2 = L1_2.stack
  L2_2 = L12_1
  L3_2 = {}
  L1_2[L2_2] = L3_2
  L1_2 = L17_1
  L1_2 = L1_2.stack
  L2_2 = L12_1
  L1_2[L2_2] = A0_2
  L1_2 = L21_1
  L1_2()
end

function L25_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2
  L3_2 = L13_1
  L3_2 = L3_2[A1_2]
  if L3_2 then
    L3_2 = L13_1
    L3_2[A1_2] = nil
  end
  L3_2 = L13_1
  L3_2[A1_2] = A2_2
  L3_2 = L0_1
  L3_2 = L3_2.startAsyncTask
  
  function L4_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3
    L0_3 = A2_2
    L1_3 = L1_1
    L1_3 = L1_3.currentTime
    L1_3 = L1_3()
    while true do
      L2_3 = L1_1
      L2_3 = L2_3.currentTime
      L2_3 = L2_3()
      L2_3 = L2_3 - L1_3
      L3_3 = L11_1
      if not (L2_3 < L3_3) then
        break
      end
      L2_3 = L0_1
      L2_3 = L2_3.sleep
      L3_3 = 0.1
      L2_3(L3_3)
    end
    L2_3 = L13_1
    L3_3 = A1_2
    L2_3 = L2_3[L3_3]
    if L2_3 == L0_3 then
      L2_3 = L13_1
      L3_3 = A1_2
      L2_3[L3_3] = nil
      L2_3 = L23_1
      L3_3 = A1_2
      L4_3 = L0_3
      L2_3(L3_3, L4_3)
    end
  end
  
  L3_2(L4_2)
end

L17_1.registerPropertyChange = L25_1

function L25_1(A0_2)
  local L1_2, L2_2
  L1_2 = L19_1
  L1_2()
  L1_2 = L24_1
  L2_2 = A0_2
  L1_2(L2_2)
end

L17_1.registerMultipleChanges = L25_1

function L25_1()
  local L0_2, L1_2, L2_2, L3_2
  L0_2 = L4_1
  L0_2 = L0_2.tableIsEmpty
  L1_2 = L17_1
  L1_2 = L1_2.stack
  L0_2 = L0_2(L1_2)
  if not L0_2 then
    L0_2 = L12_1
    if not (L0_2 < 2) then
      goto lbl_12
    end
  end
  do return end
  ::lbl_12::
  L0_2 = L12_1
  L0_2 = L0_2 - 1
  L12_1 = L0_2
  L0_2 = L17_1
  L0_2 = L0_2.stack
  L1_2 = L12_1
  L0_2 = L0_2[L1_2]
  L1_2 = NLPViewController
  L1_2 = L1_2.updateMultipleSettings
  L2_2 = L0_2
  L3_2 = properties
  L1_2(L2_2, L3_2)
  L1_2 = L21_1
  L1_2()
end

L17_1.undo = L25_1

function L25_1()
  local L0_2, L1_2, L2_2, L3_2
  L0_2 = L4_1
  L0_2 = L0_2.tableIsEmpty
  L1_2 = L17_1
  L1_2 = L1_2.stack
  L0_2 = L0_2(L1_2)
  if not L0_2 then
    L0_2 = L12_1
    L1_2 = L17_1
    L1_2 = L1_2.stack
    L1_2 = #L1_2
    if not (L0_2 >= L1_2) then
      goto lbl_15
    end
  end
  do return end
  ::lbl_15::
  L0_2 = L12_1
  L0_2 = L0_2 + 1
  L12_1 = L0_2
  L0_2 = L17_1
  L0_2 = L0_2.stack
  L1_2 = L12_1
  L0_2 = L0_2[L1_2]
  L1_2 = NLPViewController
  L1_2 = L1_2.updateMultipleSettings
  L2_2 = L0_2
  L3_2 = properties
  L1_2(L2_2, L3_2)
  L1_2 = L21_1
  L1_2()
end

L17_1.redo = L25_1

function L25_1()
  local L0_2, L1_2
  L0_2 = L17_1
  L1_2 = {}
  L0_2.stack = L1_2
  L0_2 = 0
  L12_1 = L0_2
  L0_2 = L21_1
  L0_2()
end

L17_1.handleNavigationChange = L25_1

function L25_1()
  local L0_2, L1_2
  L0_2 = L17_1
  L1_2 = {}
  L0_2.stack = L1_2
  L0_2 = L14_1
  L0_2()
end

L17_1.clearHistory = L25_1
return L17_1
