local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1, L36_1, L37_1, L38_1, L39_1, L40_1, L41_1, L42_1, L43_1, L44_1, L45_1, L46_1, L47_1, L48_1, L49_1, L50_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrView"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrFunctionContext"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrDialogs"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrLogger"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrProgressScope"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrExportSession"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrBinding"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrDevelopController"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrTasks"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrApplication"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrApplicationView"
L11_1 = L11_1(L12_1)
L12_1 = L10_1.activeCatalog
L12_1 = L12_1()
L13_1 = require
L14_1 = "NLPRolls"
L13_1 = L13_1(L14_1)
L14_1 = require
L15_1 = "UtilV3"
L14_1 = L14_1(L15_1)
L15_1 = require
L16_1 = "NegativePresets"
L15_1 = L15_1(L16_1)
L16_1 = require
L17_1 = "inspect"
L16_1 = L16_1(L17_1)
L17_1 = require
L18_1 = "NLPColorV3"
L17_1 = L17_1(L18_1)
L18_1 = require
L19_1 = "CurveEngineV3"
L18_1 = L18_1(L19_1)
L19_1 = require
L20_1 = "NLPSessionV3"
L19_1 = L19_1(L20_1)
L20_1 = require
L21_1 = "MetadataDefinitionsV3"
L20_1 = L20_1(L21_1)
L21_1 = L20_1.metadataFieldsForPhotos
L22_1 = import
L23_1 = "LrPrefs"
L22_1 = L22_1(L23_1)
L22_1 = L22_1.prefsForPlugin
L22_1 = L22_1()
L23_1 = require
L24_1 = "History"
L23_1 = L23_1(L24_1)
L24_1 = L4_1
L25_1 = "NLPViewController"
L24_1 = L24_1(L25_1)
L25_1 = _G
L25_1 = L25_1.dev
if L25_1 == true then
  L26_1 = L24_1
  L25_1 = L24_1.enable
  L27_1 = "logfile"
  L25_1(L26_1, L27_1)
else
  L26_1 = L24_1
  L25_1 = L24_1.disable
  L25_1(L26_1)
end
L25_1 = {}
NLPViewController = L25_1
L25_1 = nil

function L26_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L1_2 = _G
  L1_2 = L1_2.nlpPresets
  L1_2 = L1_2[A0_2]
  L2_2 = L1_2.settings
  L3_2 = {}
  L4_2 = pairs
  L5_2 = L2_2
  L4_2, L5_2, L6_2 = L4_2(L5_2)
  for L7_2, L8_2 in L4_2, L5_2, L6_2 do
    L9_2 = L7_2
    L10_2 = L8_2
    L11_2 = L15_1
    L11_2 = L11_2.availableSettings
    L11_2 = L11_2[L7_2]
    if L11_2 ~= nil then
      L11_2 = L15_1
      L11_2 = L11_2.availableSettings
      L11_2 = L11_2[L7_2]
      L12_2 = L11_2.internalName
      L3_2[L12_2] = L10_2
    end
  end
  return L3_2
end

function L27_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2)
  local L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2
  L6_2 = {}
  L7_2 = {}
  L8_2 = L14_1
  L8_2 = L8_2.copyTable
  L9_2 = L15_1
  L9_2 = L9_2.availableSettings
  L8_2 = L8_2(L9_2)
  L9_2 = L22_1
  L10_2 = L22_1
  L10_2 = L10_2.selectedSettings
  if not L10_2 then
    L10_2 = {}
  end
  L9_2.selectedSettings = L10_2
  L9_2 = L24_1
  L10_2 = L9_2
  L9_2 = L9_2.trace
  L11_2 = "prefs.selectedSettings: "
  L12_2 = L16_1
  L13_2 = L22_1
  L13_2 = L13_2.selectedSettings
  L12_2 = L12_2(L13_2)
  L11_2 = L11_2 .. L12_2
  L9_2(L10_2, L11_2)
  if A5_2 ~= nil then
    L9_2 = L14_1
    L9_2 = L9_2.remove_keys
    L10_2 = L8_2
    L11_2 = A5_2
    L9_2(L10_2, L11_2)
  end
  L9_2 = pairs
  L10_2 = L8_2
  L9_2, L10_2, L11_2 = L9_2(L10_2)
  for L12_2, L13_2 in L9_2, L10_2, L11_2 do
    L14_2 = L13_2.group
    L15_2 = L12_2
    L16_2 = L13_2.displayOrder
    L17_2 = L6_2[L14_2]
    if L17_2 == nil then
      L17_2 = {}
      L6_2[L14_2] = L17_2
    end
    L17_2 = L22_1
    L17_2 = L17_2.selectedSettings
    L17_2 = L17_2[L15_2]
    if L17_2 ~= nil then
      L17_2 = L22_1
      L17_2 = L17_2.selectedSettings
      L17_2 = L17_2[L15_2]
      A4_2[L15_2] = L17_2
    else
      A4_2[L15_2] = true
    end
    L17_2 = L6_2[L14_2]
    L17_2[L16_2] = L15_2
  end
  L9_2 = {}
  L10_2 = {}
  L11_2 = {}
  L12_2 = {}
  L13_2 = 1
  L14_2 = L15_1
  L14_2 = L14_2.availableGroups
  L14_2 = #L14_2
  L15_2 = 1
  for L16_2 = L13_2, L14_2, L15_2 do
    L17_2 = L15_1
    L17_2 = L17_2.availableGroups
    L17_2 = L17_2[L16_2]
    L18_2 = {}
    if L16_2 < 4 then
      L18_2 = L9_2
    elseif L16_2 < 7 then
      L18_2 = L10_2
    elseif L16_2 < 10 then
      L18_2 = L11_2
    else
      L18_2 = L12_2
    end
    L19_2 = L6_2[L17_2]
    if L19_2 ~= nil then
      L21_2 = A0_2
      L20_2 = A0_2.row
      L22_2 = {}
      L22_2.margin = 1
      L24_2 = A0_2
      L23_2 = A0_2.checkbox
      L25_2 = {}
      L25_2.title = L17_2
      L26_2 = small
      L25_2.size = L26_2
      L26_2 = L1_1
      L26_2 = L26_2.bind
      L27_2 = {}
      L27_2.keys = L19_2
      L27_2.bind_to_object = A4_2
      
      function L28_2(A0_3, A1_3, A2_3)
        local L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
        if A2_3 then
          L3_3 = "mixed"
          L4_3 = 0
          L5_3 = 1
          L6_3 = L19_2
          L6_3 = #L6_3
          L7_3 = 1
          for L8_3 = L5_3, L6_3, L7_3 do
            L9_3 = L19_2
            L9_3 = L9_3[L8_3]
            L9_3 = A1_3[L9_3]
            if L9_3 == true then
              L4_3 = L4_3 + 1
            end
          end
          L5_3 = L19_2
          L5_3 = #L5_3
          if L4_3 == L5_3 then
            L3_3 = true
          elseif L4_3 == 0 then
            L3_3 = false
          end
          return L3_3
        else
        end
      end
      
      L27_2.operation = L28_2
      
      function L28_2(A0_3, A1_3)
        local L2_3, L3_3, L4_3, L5_3, L6_3, L7_3
        if A1_3 then
          return A0_3
        else
          L2_3 = 1
          L3_3 = L19_2
          L3_3 = #L3_3
          L4_3 = 1
          for L5_3 = L2_3, L3_3, L4_3 do
            L6_3 = A4_2
            L7_3 = L19_2
            L7_3 = L7_3[L5_3]
            L6_3[L7_3] = A0_3
          end
        end
      end
      
      L27_2.transform = L28_2
      L26_2 = L26_2(L27_2)
      L25_2.value = L26_2
      L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L23_2(L24_2, L25_2)
      L22_2[1] = L23_2
      L22_2[2] = L24_2
      L22_2[3] = L25_2
      L22_2[4] = L26_2
      L22_2[5] = L27_2
      L22_2[6] = L28_2
      L22_2[7] = L29_2
      L22_2[8] = L30_2
      L22_2[9] = L31_2
      L22_2[10] = L32_2
      L22_2[11] = L33_2
      L20_2 = L20_2(L21_2, L22_2)
      L21_2 = table
      L21_2 = L21_2.insert
      L22_2 = L18_2
      L23_2 = L20_2
      L21_2(L22_2, L23_2)
      L21_2 = 1
      L22_2 = L6_2[L17_2]
      L22_2 = #L22_2
      L23_2 = 1
      for L24_2 = L21_2, L22_2, L23_2 do
        L25_2 = L6_2[L17_2]
        L25_2 = L25_2[L24_2]
        L27_2 = A0_2
        L26_2 = A0_2.row
        L28_2 = {}
        L28_2.margin_left = 20
        L28_2.margin = 1
        L30_2 = A0_2
        L29_2 = A0_2.checkbox
        L31_2 = {}
        L31_2.title = L25_2
        L32_2 = small
        L31_2.size = L32_2
        L32_2 = L1_1
        L32_2 = L32_2.bind
        L33_2 = L25_2
        L32_2 = L32_2(L33_2)
        L31_2.value = L32_2
        L29_2, L30_2, L31_2, L32_2, L33_2 = L29_2(L30_2, L31_2)
        L28_2[1] = L29_2
        L28_2[2] = L30_2
        L28_2[3] = L31_2
        L28_2[4] = L32_2
        L28_2[5] = L33_2
        L26_2 = L26_2(L27_2, L28_2)
        L27_2 = table
        L27_2 = L27_2.insert
        L28_2 = L18_2
        L29_2 = L26_2
        L27_2(L28_2, L29_2)
      end
    end
    L20_2 = table
    L20_2 = L20_2.insert
    L21_2 = L18_2
    L23_2 = A0_2
    L22_2 = A0_2.spacer
    L24_2 = {}
    L24_2.height = 17
    L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L22_2(L23_2, L24_2)
    L20_2(L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2)
  end
  L14_2 = A0_2
  L13_2 = A0_2.group_box
  L15_2 = {}
  L15_2.margin = 0
  L15_2.width = 700
  L17_2 = A0_2
  L16_2 = A0_2.row
  L18_2 = {}
  L18_2.fill_horizontal = 1
  L18_2.margin_left = 20
  L18_2.margin_right = 20
  L18_2.bind_to_object = A4_2
  L20_2 = A0_2
  L19_2 = A0_2.column
  L21_2 = {}
  L21_2.fill_horizontal = 0.25
  L23_2 = A0_2
  L22_2 = A0_2.column
  L24_2 = L9_2
  L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L22_2(L23_2, L24_2)
  L21_2[1] = L22_2
  L21_2[2] = L23_2
  L21_2[3] = L24_2
  L21_2[4] = L25_2
  L21_2[5] = L26_2
  L21_2[6] = L27_2
  L21_2[7] = L28_2
  L21_2[8] = L29_2
  L21_2[9] = L30_2
  L21_2[10] = L31_2
  L21_2[11] = L32_2
  L21_2[12] = L33_2
  L19_2 = L19_2(L20_2, L21_2)
  L21_2 = A0_2
  L20_2 = A0_2.column
  L22_2 = {}
  L22_2.fill_horizontal = 0.25
  L24_2 = A0_2
  L23_2 = A0_2.column
  L25_2 = L10_2
  L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L23_2(L24_2, L25_2)
  L22_2[1] = L23_2
  L22_2[2] = L24_2
  L22_2[3] = L25_2
  L22_2[4] = L26_2
  L22_2[5] = L27_2
  L22_2[6] = L28_2
  L22_2[7] = L29_2
  L22_2[8] = L30_2
  L22_2[9] = L31_2
  L22_2[10] = L32_2
  L22_2[11] = L33_2
  L20_2 = L20_2(L21_2, L22_2)
  L22_2 = A0_2
  L21_2 = A0_2.column
  L23_2 = {}
  L23_2.fill_horizontal = 0.25
  L25_2 = A0_2
  L24_2 = A0_2.column
  L26_2 = L11_2
  L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L24_2(L25_2, L26_2)
  L23_2[1] = L24_2
  L23_2[2] = L25_2
  L23_2[3] = L26_2
  L23_2[4] = L27_2
  L23_2[5] = L28_2
  L23_2[6] = L29_2
  L23_2[7] = L30_2
  L23_2[8] = L31_2
  L23_2[9] = L32_2
  L23_2[10] = L33_2
  L21_2 = L21_2(L22_2, L23_2)
  L23_2 = A0_2
  L22_2 = A0_2.column
  L24_2 = {}
  L24_2.fill_horizontal = 0.25
  L26_2 = A0_2
  L25_2 = A0_2.column
  L27_2 = L12_2
  L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L25_2(L26_2, L27_2)
  L24_2[1] = L25_2
  L24_2[2] = L26_2
  L24_2[3] = L27_2
  L24_2[4] = L28_2
  L24_2[5] = L29_2
  L24_2[6] = L30_2
  L24_2[7] = L31_2
  L24_2[8] = L32_2
  L24_2[9] = L33_2
  L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L22_2(L23_2, L24_2)
  L18_2[1] = L19_2
  L18_2[2] = L20_2
  L18_2[3] = L21_2
  L18_2[4] = L22_2
  L18_2[5] = L23_2
  L18_2[6] = L24_2
  L18_2[7] = L25_2
  L18_2[8] = L26_2
  L18_2[9] = L27_2
  L18_2[10] = L28_2
  L18_2[11] = L29_2
  L18_2[12] = L30_2
  L18_2[13] = L31_2
  L18_2[14] = L32_2
  L18_2[15] = L33_2
  L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2 = L16_2(L17_2, L18_2)
  L15_2[1] = L16_2
  L15_2[2] = L17_2
  L15_2[3] = L18_2
  L15_2[4] = L19_2
  L15_2[5] = L20_2
  L15_2[6] = L21_2
  L15_2[7] = L22_2
  L15_2[8] = L23_2
  L15_2[9] = L24_2
  L15_2[10] = L25_2
  L15_2[11] = L26_2
  L15_2[12] = L27_2
  L15_2[13] = L28_2
  L15_2[14] = L29_2
  L15_2[15] = L30_2
  L15_2[16] = L31_2
  L15_2[17] = L32_2
  L15_2[18] = L33_2
  L13_2 = L13_2(L14_2, L15_2)
  return L13_2
end

function L28_1(A0_2, A1_2, A2_2, A3_2, A4_2)
  local L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L6_2 = A0_2
  L5_2 = A0_2.row
  L7_2 = {}
  L9_2 = A0_2
  L8_2 = A0_2.push_button
  L10_2 = {}
  L10_2.title = "Check All"
  
  function L11_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3
    L0_3 = A4_2
    L1_3 = L0_3
    L0_3 = L0_3.pairs
    L0_3, L1_3, L2_3 = L0_3(L1_3)
    for L3_3, L4_3 in L0_3, L1_3, L2_3 do
      L5_3 = A4_2
      L5_3[L3_3] = true
    end
  end
  
  L10_2.action = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.push_button
  L11_2 = {}
  L11_2.title = "Check None"
  
  function L12_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3
    L0_3 = A4_2
    L1_3 = L0_3
    L0_3 = L0_3.pairs
    L0_3, L1_3, L2_3 = L0_3(L1_3)
    for L3_3, L4_3 in L0_3, L1_3, L2_3 do
      L5_3 = A4_2
      L5_3[L3_3] = false
    end
  end
  
  L11_2.action = L12_2
  L9_2, L10_2, L11_2, L12_2 = L9_2(L10_2, L11_2)
  L7_2[1] = L8_2
  L7_2[2] = L9_2
  L7_2[3] = L10_2
  L7_2[4] = L11_2
  L7_2[5] = L12_2
  L5_2 = L5_2(L6_2, L7_2)
  return L5_2
end

function L29_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L3_2 = A0_2
  L2_2 = A0_2.column
  L4_2 = {}
  L6_2 = A0_2
  L5_2 = A0_2.control_spacing
  L5_2 = L5_2(L6_2)
  L4_2.spacing = L5_2
  L4_2.fill_horizontal = 1
  L4_2.bind_to_object = A1_2
  L6_2 = A0_2
  L5_2 = A0_2.row
  L7_2 = {}
  L7_2.fill_horizontal = 1
  L9_2 = A0_2
  L8_2 = A0_2.static_text
  L10_2 = {}
  L10_2.title = "Preset Name:"
  L10_2.alignment = "right"
  L11_2 = L1_1
  L11_2 = L11_2.share
  L12_2 = "preset_label_width"
  L11_2 = L11_2(L12_2)
  L10_2.width = L11_2
  L8_2 = L8_2(L9_2, L10_2)
  L10_2 = A0_2
  L9_2 = A0_2.edit_field
  L11_2 = {}
  L11_2.fill_horizontal = 1
  L12_2 = L1_1
  L12_2 = L12_2.bind
  L13_2 = "presetName"
  L12_2 = L12_2(L13_2)
  L11_2.value = L12_2
  L11_2.immediate = true
  
  function L12_2(A0_3, A1_3)
    local L2_3, L3_3
    L2_3 = string
    L2_3 = L2_3.len
    L3_3 = A1_3
    L2_3 = L2_3(L3_3)
    if 0 < L2_3 then
      L2_3 = A1_2
      L2_3.validPresetName = true
      L2_3 = true
      L3_3 = A1_3
      return L2_3, L3_3
    else
      L2_3 = A1_2
      L2_3.validPresetName = false
      L2_3 = false
      L3_3 = A1_3
      return L2_3, L3_3
    end
  end
  
  L11_2.validate = L12_2
  L9_2, L10_2, L11_2, L12_2, L13_2, L14_2 = L9_2(L10_2, L11_2)
  L7_2[1] = L8_2
  L7_2[2] = L9_2
  L7_2[3] = L10_2
  L7_2[4] = L11_2
  L7_2[5] = L12_2
  L7_2[6] = L13_2
  L7_2[7] = L14_2
  L5_2 = L5_2(L6_2, L7_2)
  L7_2 = A0_2
  L6_2 = A0_2.row
  L8_2 = {}
  L8_2.fill_horizontal = 1
  L10_2 = A0_2
  L9_2 = A0_2.static_text
  L11_2 = {}
  L11_2.title = "Group:"
  L11_2.alignment = "right"
  L12_2 = L1_1
  L12_2 = L12_2.share
  L13_2 = "preset_label_width"
  L12_2 = L12_2(L13_2)
  L11_2.width = L12_2
  L9_2 = L9_2(L10_2, L11_2)
  L11_2 = A0_2
  L10_2 = A0_2.popup_menu
  L12_2 = {}
  L12_2.fill_horizontal = 1
  L13_2 = L1_1
  L13_2 = L13_2.bind
  L14_2 = "selectedPresetGroup"
  L13_2 = L13_2(L14_2)
  L12_2.value = L13_2
  L13_2 = L1_1
  L13_2 = L13_2.bind
  L14_2 = "groupDropdown"
  L13_2 = L13_2(L14_2)
  L12_2.items = L13_2
  L10_2, L11_2, L12_2, L13_2, L14_2 = L10_2(L11_2, L12_2)
  L8_2[1] = L9_2
  L8_2[2] = L10_2
  L8_2[3] = L11_2
  L8_2[4] = L12_2
  L8_2[5] = L13_2
  L8_2[6] = L14_2
  L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2 = L6_2(L7_2, L8_2)
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
  L2_2 = L2_2(L3_2, L4_2)
  return L2_2
end

function L30_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L2_2 = {}
  L2_2.separator = true
  L3_2 = {}
  L4_2 = {}
  L5_2 = _G
  L5_2 = L5_2.nlpPresets
  if L5_2 ~= nil then
    L5_2 = pairs
    L6_2 = _G
    L6_2 = L6_2.nlpPresets
    L5_2, L6_2, L7_2 = L5_2(L6_2)
    for L8_2, L9_2 in L5_2, L6_2, L7_2 do
      L10_2 = L9_2.group
      L11_2 = L9_2.isMonochrome
      L12_2 = L4_2[L10_2]
      if not L12_2 then
        L4_2[L10_2] = true
      end
    end
    L5_2 = L4_2.User
    if L5_2 == nil then
      L4_2.User = true
    end
    L5_2 = {}
    L6_2 = pairs
    L7_2 = L4_2
    L6_2, L7_2, L8_2 = L6_2(L7_2)
    for L9_2, L10_2 in L6_2, L7_2, L8_2 do
      if L10_2 == true then
        L11_2 = table
        L11_2 = L11_2.insert
        L12_2 = L5_2
        L13_2 = L9_2
        L11_2(L12_2, L13_2)
      end
    end
    L6_2 = {}
    L7_2 = 1
    L8_2 = #L5_2
    L9_2 = 1
    for L10_2 = L7_2, L8_2, L9_2 do
      L11_2 = table
      L11_2 = L11_2.insert
      L12_2 = L6_2
      L13_2 = {}
      L14_2 = L5_2[L10_2]
      L13_2.title = L14_2
      L14_2 = L5_2[L10_2]
      L13_2.value = L14_2
      L11_2(L12_2, L13_2)
    end
    if A1_2 then
      L7_2 = table
      L7_2 = L7_2.insert
      L8_2 = L6_2
      L9_2 = A1_2
      L7_2(L8_2, L9_2)
    end
    L7_2 = table
    L7_2 = L7_2.insert
    L8_2 = L6_2
    L9_2 = L2_2
    L7_2(L8_2, L9_2)
    L7_2 = table
    L7_2 = L7_2.insert
    L8_2 = L6_2
    L9_2 = {}
    L9_2.title = "New Group..."
    L9_2.value = "new_group"
    L7_2(L8_2, L9_2)
    A0_2.groupDropdown = L6_2
  end
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
  L9_2 = L1_1
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

function L32_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L1_2 = A0_2.selectedPreset
  if L1_2 ~= nil then
    L1_2 = A0_2.selectedPreset
    if L1_2 ~= "custom" then
      L1_2 = L24_1
      L2_2 = L1_2
      L1_2 = L1_2.trace
      L3_2 = "Syncing Preset Settings: "
      L4_2 = A0_2.selectedPreset
      L3_2 = L3_2 .. L4_2
      L1_2(L2_2, L3_2)
      L1_2 = A0_2.selectedPreset
      L2_2 = L24_1
      L3_2 = L2_2
      L2_2 = L2_2.trace
      L4_2 = "Going to getPresetSettings"
      L2_2(L3_2, L4_2)
      L2_2 = L26_1
      L3_2 = L1_2
      L2_2 = L2_2(L3_2)
      if L2_2 ~= nil then
        L3_2 = L24_1
        L4_2 = L3_2
        L3_2 = L3_2.trace
        L5_2 = "Preset settings: "
        L6_2 = L16_1
        L7_2 = L2_2
        L6_2 = L6_2(L7_2)
        L5_2 = L5_2 .. L6_2
        L3_2(L4_2, L5_2)
        L3_2 = pairs
        L4_2 = L2_2
        L3_2, L4_2, L5_2 = L3_2(L4_2)
        for L6_2, L7_2 in L3_2, L4_2, L5_2 do
          L8_2 = A0_2[L6_2]
          if L8_2 == nil then
            A0_2[L6_2] = L7_2
          end
        end
      end
    end
  end
  return A0_2
end

L33_1 = NLPViewController

function L34_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2
  L4_2 = L1_1
  L4_2 = L4_2.osFactory
  L4_2 = L4_2()
  A0_2 = L4_2
  L4_2 = L2_1
  L4_2 = L4_2.postAsyncTaskWithContext
  L5_2 = "syncSettingsDialog"
  
  function L6_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3
    L1_3 = L7_1
    L1_3 = L1_3.makePropertyTable
    L2_3 = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = "Sync Settings"
    L3_3 = L27_1
    L4_3 = A0_2
    L5_3 = A1_2
    L6_3 = A2_2
    L7_3 = A3_2
    L8_3 = L1_3
    L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3, L8_3)
    L4_3 = L28_1
    L5_3 = A0_2
    L6_3 = A1_2
    L7_3 = A2_2
    L8_3 = A3_2
    L9_3 = L1_3
    L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3, L9_3)
    L5_3 = L3_1
    L5_3 = L5_3.presentModalDialog
    L6_3 = {}
    L6_3.title = "Sync Settings"
    L6_3.contents = L3_3
    L6_3.accessoryView = L4_3
    L6_3.actionVerb = "Sync"
    L5_3 = L5_3(L6_3)
    if L5_3 == "ok" then
      L6_3 = L22_1
      L7_3 = {}
      L6_3.selectedSettings = L7_3
      L7_3 = L1_3
      L6_3 = L1_3.pairs
      L6_3, L7_3, L8_3 = L6_3(L7_3)
      for L9_3, L10_3 in L6_3, L7_3, L8_3 do
        L11_3 = L22_1
        L11_3 = L11_3.selectedSettings
        L11_3[L9_3] = L10_3
      end
      L6_3 = L22_1
      L7_3 = L22_1
      L7_3 = L7_3.selectedSettings
      L6_3.selectedSettings = L7_3
      L6_3 = {}
      L8_3 = L1_3
      L7_3 = L1_3.pairs
      L7_3, L8_3, L9_3 = L7_3(L8_3)
      for L10_3, L11_3 in L7_3, L8_3, L9_3 do
        if L11_3 == true then
          L12_3 = L15_1
          L12_3 = L12_3.availableSettings
          L12_3 = L12_3[L10_3]
          L12_3 = L12_3.internalName
          L13_3 = A1_2
          L13_3 = L13_3[L12_3]
          L14_3 = A1_2
          L14_3 = L14_3[L12_3]
          L6_3[L12_3] = L14_3
        end
      end
      L7_3 = L32_1
      L8_3 = L6_3
      L7_3 = L7_3(L8_3)
      L6_3 = L7_3
      L7_3 = NLPViewController
      L7_3 = L7_3.syncPhotoSettings
      L8_3 = L6_3
      L9_3 = A2_2
      L7_3(L8_3, L9_3)
    else
    end
  end
  
  L4_2(L5_2, L6_2)
end

L33_1.loadSyncSettingsDialog = L34_1
L33_1 = NLPViewController

function L34_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2
  L4_2 = L1_1
  L4_2 = L4_2.osFactory
  L4_2 = L4_2()
  A0_2 = L4_2
  L4_2 = L2_1
  L4_2 = L4_2.postAsyncTaskWithContext
  L5_2 = "syncSettingsDialog"
  
  function L6_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3
    L1_3 = L7_1
    L1_3 = L1_3.makePropertyTable
    L2_3 = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = "Copy Settings"
    L3_3 = L27_1
    L4_3 = A0_2
    L5_3 = A1_2
    L6_3 = A2_2
    L7_3 = A3_2
    L8_3 = L1_3
    L3_3 = L3_3(L4_3, L5_3, L6_3, L7_3, L8_3)
    L4_3 = L28_1
    L5_3 = A0_2
    L6_3 = A1_2
    L7_3 = A2_2
    L8_3 = A3_2
    L9_3 = L1_3
    L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3, L9_3)
    L5_3 = L3_1
    L5_3 = L5_3.presentModalDialog
    L6_3 = {}
    L6_3.title = "Copy Settings"
    L6_3.contents = L3_3
    L6_3.accessoryView = L4_3
    L6_3.actionVerb = "Copy"
    L5_3 = L5_3(L6_3)
    if L5_3 == "ok" then
      L6_3 = L22_1
      L7_3 = {}
      L6_3.selectedSettings = L7_3
      L7_3 = L1_3
      L6_3 = L1_3.pairs
      L6_3, L7_3, L8_3 = L6_3(L7_3)
      for L9_3, L10_3 in L6_3, L7_3, L8_3 do
        L11_3 = L22_1
        L11_3 = L11_3.selectedSettings
        L11_3[L9_3] = L10_3
      end
      L6_3 = L22_1
      L7_3 = L22_1
      L7_3 = L7_3.selectedSettings
      L6_3.selectedSettings = L7_3
      L6_3 = _G
      L7_3 = {}
      L6_3.copySettings = L7_3
      L6_3 = L22_1
      L7_3 = {}
      L6_3.copySettings = L7_3
      L7_3 = L1_3
      L6_3 = L1_3.pairs
      L6_3, L7_3, L8_3 = L6_3(L7_3)
      for L9_3, L10_3 in L6_3, L7_3, L8_3 do
        if L10_3 == true then
          L11_3 = L15_1
          L11_3 = L11_3.availableSettings
          L11_3 = L11_3[L9_3]
          L11_3 = L11_3.internalName
          L12_3 = A1_2
          L12_3 = L12_3[L11_3]
          L13_3 = _G
          L13_3 = L13_3.copySettings
          L14_3 = A1_2
          L14_3 = L14_3[L11_3]
          L13_3[L11_3] = L14_3
          L13_3 = L22_1
          L13_3 = L13_3.copySettings
          L14_3 = A1_2
          L14_3 = L14_3[L11_3]
          L13_3[L11_3] = L14_3
        end
      end
      L6_3 = L22_1
      L7_3 = L22_1
      L7_3 = L7_3.copySettings
      L6_3.copySettings = L7_3
    else
    end
  end
  
  L4_2(L5_2, L6_2)
end

L33_1.loadCopySettingsDialog = L34_1
L33_1 = NLPViewController

function L34_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = _G
  L2_2 = L2_2.copySettings
  if not L2_2 then
    L2_2 = L22_1
    L2_2 = L2_2.copySettings
  end
  if L2_2 == nil then
    L3_2 = "Nothing to Paste"
    L4_2 = "You must first copy settings in order to use paste."
    L5_2 = "OK"
    L6_2 = L3_1
    L6_2 = L6_2.messageWithDoNotShow
    L7_2 = {}
    L7_2.message = L3_2
    L7_2.info = L4_2
    L7_2.actionPrefKey = L5_2
    L6_2(L7_2)
    return
  end
  L3_2 = L32_1
  L4_2 = L2_2
  L3_2 = L3_2(L4_2)
  L2_2 = L3_2
  L3_2 = NLPViewController
  L3_2 = L3_2.updateMultipleSettings
  L4_2 = L2_2
  L5_2 = A0_2
  L6_2 = true
  L3_2(L4_2, L5_2, L6_2)
end

L33_1.pasteSettings = L34_1
L33_1 = NLPViewController

function L34_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2
  L4_2 = {}
  L5_2 = "Preset Settings"
  L4_2[1] = L5_2
  L5_2 = L1_1
  L5_2 = L5_2.osFactory
  L5_2 = L5_2()
  f = L5_2
  L5_2 = L2_1
  L5_2 = L5_2.postAsyncTaskWithContext
  L6_2 = "syncSettingsDialog"
  
  function L7_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3, L16_3, L17_3
    L1_3 = L7_1
    L1_3 = L1_3.makePropertyTable
    L2_3 = A0_3
    L1_3 = L1_3(L2_3)
    L2_3 = L7_1
    L2_3 = L2_3.makePropertyTable
    L3_3 = A0_3
    L2_3 = L2_3(L3_3)
    L2_3.validPresetName = false
    L3_3 = ""
    L4_3 = ""
    L5_3 = nil
    L6_3 = A0_2
    if L6_3 == "save" then
      L3_3 = "Create New Preset"
      L4_3 = "Save Preset"
      L2_3.selectedPresetGroup = "User"
      L7_3 = L2_3
      L6_3 = L2_3.addObserver
      L8_3 = "selectedPresetGroup"
      
      function L9_3(A0_4, A1_4, A2_4)
        local L3_4, L4_4, L5_4, L6_4, L7_4, L8_4
        if A2_4 ~= "new_group" then
          return
        end
        L3_4 = L3_1
        L3_4 = L3_4.presentModalDialog
        L4_4 = {}
        L4_4.title = "New Group"
        L5_4 = L31_1
        L6_4 = f
        L7_4 = A0_4
        L5_4 = L5_4(L6_4, L7_4)
        L4_4.contents = L5_4
        L4_4.actionVerb = "Create"
        L5_4 = {}
        L6_4 = {}
        L6_4.bind_to_object = A0_4
        L6_4.key = "validGroupName"
        L5_4.enabled = L6_4
        L4_4.actionBinding = L5_4
        L3_4 = L3_4(L4_4)
        if L3_4 == "ok" then
          L4_4 = A0_4.groupDropdown
          L4_4 = #L4_4
          L4_4 = L4_4 - 1
          L5_4 = A0_4.newGroupName
          L6_4 = L30_1
          L7_4 = A0_4
          L8_4 = {}
          L8_4.title = L5_4
          L8_4.value = L5_4
          L6_4(L7_4, L8_4)
          A0_4.selectedPresetGroup = L5_4
        end
      end
      
      L6_3(L7_3, L8_3, L9_3)
      L6_3 = L30_1
      L7_3 = L2_3
      L6_3(L7_3)
      L6_3 = L29_1
      L7_3 = f
      L8_3 = L2_3
      L6_3 = L6_3(L7_3, L8_3)
      L7_3 = L27_1
      L8_3 = f
      L9_3 = A1_2
      L10_3 = A2_2
      L11_3 = A3_2
      L12_3 = L1_3
      L13_3 = L4_2
      L7_3 = L7_3(L8_3, L9_3, L10_3, L11_3, L12_3, L13_3)
      L8_3 = f
      L9_3 = L8_3
      L8_3 = L8_3.view
      L10_3 = {}
      L11_3 = L6_3
      L12_3 = f
      L13_3 = L12_3
      L12_3 = L12_3.spacer
      L14_3 = {}
      L14_3.height = 10
      L12_3 = L12_3(L13_3, L14_3)
      L13_3 = L7_3
      L10_3[1] = L11_3
      L10_3[2] = L12_3
      L10_3[3] = L13_3
      L8_3 = L8_3(L9_3, L10_3)
      L5_3 = L8_3
    else
      L6_3 = A0_2
      if L6_3 == "update" then
        L6_3 = "Update \""
        L7_3 = _G
        L7_3 = L7_3.nlpPresets
        L8_3 = lastSelectedPreset
        L7_3 = L7_3[L8_3]
        L7_3 = L7_3.presetName
        L8_3 = "\""
        L3_3 = L6_3 .. L7_3 .. L8_3
        L2_3.validPresetName = true
        L4_3 = "Update Preset"
        L6_3 = L27_1
        L7_3 = f
        L8_3 = A1_2
        L9_3 = A2_2
        L10_3 = A3_2
        L11_3 = L1_3
        L12_3 = L4_2
        L6_3 = L6_3(L7_3, L8_3, L9_3, L10_3, L11_3, L12_3)
        L5_3 = L6_3
      end
    end
    L6_3 = L28_1
    L7_3 = f
    L8_3 = A1_2
    L9_3 = A2_2
    L10_3 = A3_2
    L11_3 = L1_3
    L6_3 = L6_3(L7_3, L8_3, L9_3, L10_3, L11_3)
    L7_3 = L3_1
    L7_3 = L7_3.presentModalDialog
    L8_3 = {}
    L8_3.title = L3_3
    L8_3.contents = L5_3
    L8_3.accessoryView = L6_3
    L9_3 = {}
    L10_3 = {}
    L10_3.bind_to_object = L2_3
    L10_3.key = "validPresetName"
    L9_3.enabled = L10_3
    L8_3.actionBinding = L9_3
    L8_3.actionVerb = L4_3
    L7_3 = L7_3(L8_3)
    if L7_3 == "ok" then
      L8_3 = L22_1
      L9_3 = {}
      L8_3.selectedSettings = L9_3
      L9_3 = L1_3
      L8_3 = L1_3.pairs
      L8_3, L9_3, L10_3 = L8_3(L9_3)
      for L11_3, L12_3 in L8_3, L9_3, L10_3 do
        L13_3 = L22_1
        L13_3 = L13_3.selectedSettings
        L13_3[L11_3] = L12_3
      end
      L8_3 = L22_1
      L9_3 = L22_1
      L9_3 = L9_3.selectedSettings
      L8_3.selectedSettings = L9_3
      L8_3 = {}
      L10_3 = L1_3
      L9_3 = L1_3.pairs
      L9_3, L10_3, L11_3 = L9_3(L10_3)
      for L12_3, L13_3 in L9_3, L10_3, L11_3 do
        L14_3 = L12_3
        if L13_3 == true then
          L15_3 = L15_1
          L15_3 = L15_3.availableSettings
          L15_3 = L15_3[L12_3]
          L15_3 = L15_3.internalName
          L16_3 = A1_2
          L16_3 = L16_3[L15_3]
          L17_3 = A1_2
          L17_3 = L17_3[L15_3]
          L8_3[L14_3] = L17_3
        end
      end
      L9_3 = A0_2
      if L9_3 == "save" then
        L9_3 = L14_1
        L9_3 = L9_3.generateUUID
        L9_3 = L9_3()
        L10_3 = L2_3.presetName
        L11_3 = L14_1
        L11_3 = L11_3.sanitize_filename
        L12_3 = L10_3
        L11_3 = L11_3(L12_3)
        L12_3 = ".json"
        L11_3 = L11_3 .. L12_3
        L12_3 = _G
        L12_3 = L12_3.nlpPresets
        L13_3 = {}
        L13_3.presetName = L10_3
        L13_3.id = L9_3
        L14_3 = L2_3.selectedPresetGroup
        L13_3.group = L14_3
        L13_3.settings = L8_3
        L13_3.needsUpdate = true
        L13_3.fileName = L11_3
        L14_3 = _G
        L14_3 = L14_3.isMonochrome
        if not L14_3 then
          L14_3 = false
        end
        L13_3.isMonochrome = L14_3
        L14_3 = _G
        L14_3 = L14_3.isPositive
        if not L14_3 then
          L14_3 = false
        end
        L13_3.isPositive = L14_3
        L12_3[L9_3] = L13_3
        L12_3 = createPresetTable
        L12_3()
        L12_3 = A1_2
        L12_3.selectedPreset = L9_3
      else
        L9_3 = A0_2
        if L9_3 == "update" then
          L9_3 = _G
          L9_3 = L9_3.nlpPresets
          L10_3 = lastSelectedPreset
          L9_3 = L9_3[L10_3]
          L9_3.settings = L8_3
          L9_3 = _G
          L9_3 = L9_3.nlpPresets
          L10_3 = lastSelectedPreset
          L9_3 = L9_3[L10_3]
          L9_3.needsUpdate = true
          L9_3 = A1_2
          L10_3 = lastSelectedPreset
          L9_3.selectedPreset = L10_3
        end
      end
    else
      L8_3 = A1_2
      L9_3 = L8_3
      L8_3 = L8_3.removeObserver
      L10_3 = "selectedPreset"
      L11_3 = propertiesObserverTable
      L8_3(L9_3, L10_3, L11_3)
      L8_3 = A1_2
      L9_3 = lastSelectedPreset
      L8_3.selectedPreset = L9_3
      L8_3 = A1_2
      L9_3 = L8_3
      L8_3 = L8_3.addObserver
      L10_3 = "selectedPreset"
      L11_3 = propertiesObserverTable
      
      function L12_3(A0_4, A1_4, A2_4, A3_4)
        local L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4
        L4_4 = NLPViewController
        L4_4 = L4_4.propertiesChanged
        L5_4 = A0_4
        L6_4 = A1_4
        L7_4 = A2_4
        L8_4 = A3_4
        L9_4 = A2_2
        L10_4 = A3_2
        L4_4(L5_4, L6_4, L7_4, L8_4, L9_4, L10_4)
      end
      
      L8_3(L9_3, L10_3, L11_3, L12_3)
    end
  end
  
  L5_2(L6_2, L7_2)
end

L33_1.loadPresetDialog = L34_1
L33_1 = nil
L34_1 = nil
L35_1 = nil
L36_1 = false
L37_1 = false
L38_1 = 0.5
L39_1 = NLPViewController

function L40_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2
  L3_2 = L24_1
  L4_2 = L3_2
  L3_2 = L3_2.trace
  L5_2 = "In applySettingsWithDelay"
  L3_2(L4_2, L5_2)
  L33_1 = A0_2
  L34_1 = A1_2
  L35_1 = A2_2
  L3_2 = L36_1
  if L3_2 == false then
    L3_2 = L9_1
    L3_2 = L3_2.startAsyncTask
    
    function L4_2()
      local L0_3, L1_3, L2_3, L3_3
      L0_3 = true
      L36_1 = L0_3
      L0_3 = L24_1
      L1_3 = L0_3
      L0_3 = L0_3.trace
      L2_3 = "Applying settings immediately"
      L0_3(L1_3, L2_3)
      L0_3 = NLPViewController
      L0_3 = L0_3.applySettingsWithWriteAccess
      L1_3 = L33_1
      L2_3 = L34_1
      L3_3 = L35_1
      L0_3(L1_3, L2_3, L3_3)
      L0_3 = L24_1
      L1_3 = L0_3
      L0_3 = L0_3.trace
      L2_3 = "Return from applying settings"
      L0_3(L1_3, L2_3)
      L0_3 = L9_1
      L0_3 = L0_3.sleep
      L1_3 = L38_1
      L0_3(L1_3)
      L0_3 = L24_1
      L1_3 = L0_3
      L0_3 = L0_3.trace
      L2_3 = "Return from sleep"
      L0_3(L1_3, L2_3)
      L0_3 = L37_1
      if L0_3 == true then
        L0_3 = L24_1
        L1_3 = L0_3
        L0_3 = L0_3.trace
        L2_3 = "Applying settings in queue"
        L0_3(L1_3, L2_3)
        L0_3 = NLPViewController
        L0_3 = L0_3.applySettingsWithWriteAccess
        L1_3 = L33_1
        L2_3 = L34_1
        L3_3 = L35_1
        L0_3(L1_3, L2_3, L3_3)
        L0_3 = false
        L37_1 = L0_3
      end
      L0_3 = false
      L36_1 = L0_3
      L0_3 = L24_1
      L1_3 = L0_3
      L0_3 = L0_3.trace
      L2_3 = "Timer reset"
      L0_3(L1_3, L2_3)
    end
    
    L3_2(L4_2)
  else
    L3_2 = L24_1
    L4_2 = L3_2
    L3_2 = L3_2.trace
    L5_2 = "Add settings to queue"
    L3_2(L4_2, L5_2)
    L3_2 = true
    L37_1 = L3_2
  end
end

L39_1.applySettingsWithDelay = L40_1
L39_1 = NLPViewController

function L40_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L3_2 = L24_1
  L4_2 = L3_2
  L3_2 = L3_2.trace
  L5_2 = "APPLYING SETTING WITH WRITE ACCESS"
  L3_2(L4_2, L5_2)
  L3_2 = L12_1
  L3_2 = L3_2.hasWriteAccess
  if L3_2 ~= true then
    L3_2 = L24_1
    L4_2 = L3_2
    L3_2 = L3_2.trace
    L5_2 = "Catalog is not being writen. Process with applying settings"
    L3_2(L4_2, L5_2)
    L3_2 = L8_1
    L3_2 = L3_2.stopTracking
    L3_2()
    L3_2 = nil
    L4_2 = L12_1
    L5_2 = L4_2
    L4_2 = L4_2.withWriteAccessDo
    L6_2 = "Apply Negative Settings"
    
    function L7_2(A0_3)
      local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
      L1_3 = A2_2
      if L1_3 ~= nil then
        L1_3 = 1
        L2_3 = A2_2
        L2_3 = #L2_3
        L3_3 = 1
        for L4_3 = L1_3, L2_3, L3_3 do
          L5_3 = A2_2
          L5_3 = L5_3[L4_3]
          L5_3 = L5_3.photo
          L6_3 = A2_2
          L6_3 = L6_3[L4_3]
          L6_3 = L6_3.settingsForPhoto
          L8_3 = L5_3
          L7_3 = L5_3.applyDevelopSettings
          L9_3 = L6_3
          L7_3(L8_3, L9_3)
        end
      else
        L1_3 = A1_2
        if L1_3 == nil then
          L1_3 = _G
          L1_3 = L1_3.activePhotos
          L2_3 = _G
          L2_3 = L2_3.activePhotoIndex
          L1_3 = L1_3[L2_3]
          L3_2 = L1_3
        else
          L1_3 = _G
          L1_3 = L1_3.activePhotos
          L2_3 = A1_2
          L1_3 = L1_3[L2_3]
          L3_2 = L1_3
        end
        L1_3 = L3_2
        L2_3 = L1_3
        L1_3 = L1_3.applyDevelopSettings
        L3_3 = A0_2
        L1_3(L2_3, L3_3)
      end
    end
    
    L8_2 = {}
    L8_2.timeout = 20
    L4_2 = L4_2(L5_2, L6_2, L7_2, L8_2)
    L5_2 = L24_1
    L6_2 = L5_2
    L5_2 = L5_2.trace
    L7_2 = "Catalog result: "
    L8_2 = L4_2
    L7_2 = L7_2 .. L8_2
    L5_2(L6_2, L7_2)
    L5_2 = undoCount
    L5_2 = L5_2 + 1
    undoCount = L5_2
  else
    L3_2 = L24_1
    L4_2 = L3_2
    L3_2 = L3_2.trace
    L5_2 = "Catalog is being written. Sleep and try again"
    L3_2(L4_2, L5_2)
    L3_2 = L9_1
    L3_2 = L3_2.sleep
    L4_2 = 0.1
    L3_2(L4_2)
    L3_2 = NLPViewController
    L3_2 = L3_2.applySettingsWithWriteAccess
    L4_2 = A0_2
    L5_2 = A1_2
    L6_2 = A2_2
    L3_2(L4_2, L5_2, L6_2)
  end
end

L39_1.applySettingsWithWriteAccess = L40_1
L39_1 = NLPViewController

function L40_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L4_2 = L24_1
  L5_2 = L4_2
  L4_2 = L4_2.trace
  L6_2 = "In applyLightroomSettings"
  L4_2(L5_2, L6_2)
  if A3_2 == nil then
    L4_2 = L11_1
    L4_2 = L4_2.getCurrentModuleName
    L4_2 = L4_2()
    if L4_2 == "develop" and A2_2 ~= true then
      L4_2 = L24_1
      L5_2 = L4_2
      L4_2 = L4_2.trace
      L6_2 = "Applying settings directly"
      L4_2(L5_2, L6_2)
      L4_2 = pairs
      L5_2 = A0_2
      L4_2, L5_2, L6_2 = L4_2(L5_2)
      for L7_2, L8_2 in L4_2, L5_2, L6_2 do
        L9_2 = L8_1
        L9_2 = L9_2.setValue
        L10_2 = L7_2
        L11_2 = L8_2
        L9_2(L10_2, L11_2)
      end
  end
  elseif A3_2 ~= nil then
    L4_2 = NLPViewController
    L4_2 = L4_2.applySettingsWithDelay
    L5_2 = A0_2
    L6_2 = A1_2
    L7_2 = A3_2
    L4_2(L5_2, L6_2, L7_2)
  else
    L4_2 = L11_1
    L4_2 = L4_2.getCurrentModuleName
    L4_2 = L4_2()
    if L4_2 ~= "library" then
      L4_2 = L11_1
      L4_2 = L4_2.getCurrentModuleName
      L4_2 = L4_2()
      if L4_2 ~= "develop" then
        goto lbl_62
      end
    end
    L4_2 = L24_1
    L5_2 = L4_2
    L4_2 = L4_2.trace
    L6_2 = "Applying settings with delay"
    L4_2(L5_2, L6_2)
    L4_2 = NLPViewController
    L4_2 = L4_2.applySettingsWithDelay
    L5_2 = A0_2
    L6_2 = A1_2
    L4_2(L5_2, L6_2)
    L4_2 = undoCount
    L4_2 = L4_2 + 1
    undoCount = L4_2
    goto lbl_66
    ::lbl_62::
    L4_2 = L3_1
    L4_2 = L4_2.message
    L5_2 = "To use this plugin, you must be in either the develop module or the library module"
    L4_2(L5_2)
  end
  ::lbl_66::
end

L39_1.applyLightroomSettings = L40_1
L39_1 = NLPViewController

function L40_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = "WARNING!"
  L2_2 = "'Match' (previously 'Sync Scene') uses the image analysis from the current image and applies it to the other selected images. This will overwrite any existing analysis on the other images. Are you sure you want to continue?"
  L3_2 = "matchWarning"
  L4_2 = L3_1
  L4_2 = L4_2.promptForActionWithDoNotShow
  L5_2 = {}
  L5_2.message = L1_2
  L5_2.info = L2_2
  L5_2.actionPrefKey = L3_2
  L6_2 = {}
  L7_2 = {}
  L7_2.label = "Yes, continue"
  L7_2.verb = "ok"
  L6_2[1] = L7_2
  L5_2.verbBtns = L6_2
  L4_2 = L4_2(L5_2)
  if L4_2 == "ok" then
    L5_2 = _G
    L5_2 = L5_2.activePhotosMeta
    L6_2 = _G
    L6_2 = L6_2.activePhotoIndex
    L5_2 = L5_2[L6_2]
    L5_2 = L5_2.redChannel
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L7_2 = _G
    L7_2 = L7_2.activePhotoIndex
    L6_2 = L6_2[L7_2]
    L6_2 = L6_2.greenChannel
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L8_2 = _G
    L8_2 = L8_2.activePhotoIndex
    L7_2 = L7_2[L8_2]
    L7_2 = L7_2.blueChannel
    L8_2 = _G
    L8_2 = L8_2.activePhotosMeta
    L9_2 = _G
    L9_2 = L9_2.activePhotoIndex
    L8_2 = L8_2[L9_2]
    L8_2 = L8_2.autoWB
    L9_2 = L9_1
    L9_2 = L9_2.startAsyncTask
    
    function L10_2()
      local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3, L13_3, L14_3, L15_3
      L0_3 = L8_1
      L0_3 = L0_3.stopTracking
      L0_3()
      L0_3 = _G
      L0_3 = L0_3.activePhotos
      L1_3 = _G
      L1_3 = L1_3.activePhotoIndex
      L0_3 = L0_3[L1_3]
      L1_3 = L0_3
      L0_3 = L0_3.getDevelopSettings
      L0_3 = L0_3(L1_3)
      L1_3 = {}
      L2_3 = L0_3.Look
      L1_3.Look = L2_3
      L2_3 = L0_3.CameraProfile
      L1_3.CameraProfile = L2_3
      L2_3 = L0_3.ConvertToGrayscale
      L1_3.ConvertToGrayscale = L2_3
      L2_3 = L0_3.ShadowTint
      L1_3.ShadowTint = L2_3
      L2_3 = L0_3.RedHue
      L1_3.RedHue = L2_3
      L2_3 = L0_3.RedSaturation
      L1_3.RedSaturation = L2_3
      L2_3 = L0_3.GreenHue
      L1_3.GreenHue = L2_3
      L2_3 = L0_3.GreenSaturation
      L1_3.GreenSaturation = L2_3
      L2_3 = L0_3.BlueHue
      L1_3.BlueHue = L2_3
      L2_3 = L0_3.BlueSaturation
      L1_3.BlueSaturation = L2_3
      L2_3 = L0_3.LensProfileEnable
      L1_3.LensProfileEnable = L2_3
      L2_3 = L0_3.SaturationAdjustmentRed
      L1_3.SaturationAdjustmentRed = L2_3
      L2_3 = L0_3.SaturationAdjustmentOrange
      L1_3.SaturationAdjustmentOrange = L2_3
      L2_3 = L0_3.SaturationAdjustmentYellow
      L1_3.SaturationAdjustmentYellow = L2_3
      L2_3 = L0_3.SaturationAdjustmentGreen
      L1_3.SaturationAdjustmentGreen = L2_3
      L2_3 = L0_3.SaturationAdjustmentAqua
      L1_3.SaturationAdjustmentAqua = L2_3
      L2_3 = L0_3.SaturationAdjustmentBlue
      L1_3.SaturationAdjustmentBlue = L2_3
      L2_3 = L0_3.SaturationAdjustmentPurple
      L1_3.SaturationAdjustmentPurple = L2_3
      L2_3 = L0_3.SaturationAdjustmentMagenta
      L1_3.SaturationAdjustmentMagenta = L2_3
      L2_3 = L0_3.ToneCurvePV2012Red
      L1_3.ToneCurvePV2012Red = L2_3
      L2_3 = L0_3.ToneCurvePV2012Blue
      L1_3.ToneCurvePV2012Blue = L2_3
      L2_3 = L0_3.ToneCurvePV2012Green
      L1_3.ToneCurvePV2012Green = L2_3
      L2_3 = L0_3.ToneCurvePV2012
      L1_3.ToneCurvePV2012 = L2_3
      L2_3 = L0_3.ColorNoiseReduction
      L1_3.ColorNoiseReduction = L2_3
      L2_3 = L0_3.ColorNoiseReductionDetail
      L1_3.ColorNoiseReductionDetail = L2_3
      L2_3 = L0_3.ColorNoiseReductionSmoothness
      L1_3.ColorNoiseReductionSmoothness = L2_3
      L2_3 = L0_3.LuminanceNoiseReductionContrast
      L1_3.LuminanceNoiseReductionContrast = L2_3
      L2_3 = L0_3.LuminanceNoiseReductionDetail
      L1_3.LuminanceNoiseReductionDetail = L2_3
      L2_3 = L0_3.LuminanceSmoothing
      L1_3.LuminanceSmoothing = L2_3
      L2_3 = L0_3.SharpenDetail
      L1_3.SharpenDetail = L2_3
      L2_3 = L0_3.SharpenEdgeMasking
      L1_3.SharpenEdgeMasking = L2_3
      L2_3 = L0_3.SharpenRadius
      L1_3.SharpenRadius = L2_3
      L2_3 = L0_3.Sharpness
      L1_3.Sharpness = L2_3
      L2_3 = L0_3.SplitToningShadowHue
      L1_3.SplitToningShadowHue = L2_3
      L2_3 = L0_3.SplitToningShadowSaturation
      L1_3.SplitToningShadowSaturation = L2_3
      L2_3 = L0_3.SplitToningBalance
      L1_3.SplitToningBalance = L2_3
      L2_3 = L0_3.SplitToningHighlightHue
      L1_3.SplitToningHighlightHue = L2_3
      L2_3 = L0_3.SplitToningHighlightSaturation
      L1_3.SplitToningHighlightSaturation = L2_3
      L2_3 = L0_3.MaskGroupBasedCorrections
      if L2_3 ~= nil then
        L2_3 = L0_3.MaskGroupBasedCorrections
        L1_3.MaskGroupBasedCorrections = L2_3
      end
      L2_3 = nil
      L3_3 = nil
      L4_3 = nil
      L5_3 = nil
      L6_3 = _G
      L6_3 = L6_3.activePhotosMeta
      L7_3 = _G
      L7_3 = L7_3.activePhotoIndex
      L6_3 = L6_3[L7_3]
      L6_3 = L6_3.fileFormat
      if L6_3 ~= "RAW" then
        L6_3 = _G
        L6_3 = L6_3.activePhotosMeta
        L7_3 = _G
        L7_3 = L7_3.activePhotoIndex
        L6_3 = L6_3[L7_3]
        L6_3 = L6_3.fileFormat
        if L6_3 ~= "DNG" then
          goto lbl_115
        end
      end
      L2_3 = L0_3.Temperature
      L3_3 = L0_3.Tint
      L4_3 = "Temperature"
      L5_3 = "Tint"
      goto lbl_119
      ::lbl_115::
      L2_3 = L0_3.IncrementalTemperature
      L3_3 = L0_3.IncrementalTint
      L4_3 = "IncrementalTemperature"
      L5_3 = "IncrementalTint"
      ::lbl_119::
      L6_3 = L0_3.WhiteBalance
      L1_3.WhiteBalance = L6_3
      L1_3[L4_3] = L2_3
      L1_3[L5_3] = L3_3
      L6_3 = {}
      L7_3 = L14_1
      L7_3 = L7_3.copyObservableTable
      L8_3 = A0_2
      L7_3 = L7_3(L8_3)
      L8_3 = 1
      L9_3 = _G
      L9_3 = L9_3.activePhotos
      L9_3 = #L9_3
      L10_3 = 1
      for L11_3 = L8_3, L9_3, L10_3 do
        L12_3 = _G
        L12_3 = L12_3.activePhotoIndex
        if L11_3 ~= L12_3 then
          L12_3 = _G
          L12_3 = L12_3.activePhotosMeta
          L12_3 = L12_3[L11_3]
          L13_3 = _G
          L13_3 = L13_3.activePhotosMeta
          L14_3 = _G
          L14_3 = L14_3.activePhotoIndex
          L13_3 = L13_3[L14_3]
          L13_3 = L13_3.autoWB
          L12_3.autoWB = L13_3
          L12_3 = _G
          L12_3 = L12_3.activePhotosMeta
          L12_3 = L12_3[L11_3]
          L13_3 = _G
          L13_3 = L13_3.activePhotosMeta
          L14_3 = _G
          L14_3 = L14_3.activePhotoIndex
          L13_3 = L13_3[L14_3]
          L13_3 = L13_3.redChannel
          L12_3.redChannel = L13_3
          L12_3 = _G
          L12_3 = L12_3.activePhotosMeta
          L12_3 = L12_3[L11_3]
          L13_3 = _G
          L13_3 = L13_3.activePhotosMeta
          L14_3 = _G
          L14_3 = L14_3.activePhotoIndex
          L13_3 = L13_3[L14_3]
          L13_3 = L13_3.greenChannel
          L12_3.greenChannel = L13_3
          L12_3 = _G
          L12_3 = L12_3.activePhotosMeta
          L12_3 = L12_3[L11_3]
          L13_3 = _G
          L13_3 = L13_3.activePhotosMeta
          L14_3 = _G
          L14_3 = L14_3.activePhotoIndex
          L13_3 = L13_3[L14_3]
          L13_3 = L13_3.blueChannel
          L12_3.blueChannel = L13_3
          L12_3 = _G
          L12_3 = L12_3.activePhotosMeta
          L12_3 = L12_3[L11_3]
          L12_3.settings = L7_3
          L12_3 = {}
          L13_3 = _G
          L13_3 = L13_3.activePhotos
          L13_3 = L13_3[L11_3]
          L12_3.photo = L13_3
          L12_3.settingsForPhoto = L1_3
          L13_3 = table
          L13_3 = L13_3.insert
          L14_3 = L6_3
          L15_3 = L12_3
          L13_3(L14_3, L15_3)
        end
      end
      L8_3 = L24_1
      L9_3 = L8_3
      L8_3 = L8_3.trace
      L10_3 = "Sync scene - apply settings"
      L8_3(L9_3, L10_3)
      L8_3 = NLPViewController
      L8_3 = L8_3.applyLightroomSettings
      L9_3 = L1_3
      L10_3 = nil
      L11_3 = true
      L12_3 = L6_3
      L8_3(L9_3, L10_3, L11_3, L12_3)
    end
    
    L9_2(L10_2)
  end
end

L39_1.syncScene = L40_1
L39_1 = false
L40_1 = {}
L41_1 = nil
L42_1 = nil
L43_1 = 2

function L44_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2
  L2_2 = L17_1
  L3_2 = A0_2.colorPicker
  L4_2 = L3_2
  L3_2 = L3_2.red
  L3_2 = L3_2(L4_2)
  L4_2 = A0_2.colorPicker
  L5_2 = L4_2
  L4_2 = L4_2.green
  L4_2 = L4_2(L5_2)
  L5_2 = A0_2.colorPicker
  L6_2 = L5_2
  L5_2 = L5_2.blue
  L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2 = L5_2(L6_2)
  L2_2 = L2_2(L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2)
  L3_2 = WIN_ENV
  if L3_2 then
    L3_2 = L39_1
    if L3_2 == false then
      L3_2 = true
      L39_1 = L3_2
      L3_2 = table
      L3_2 = L3_2.insert
      L4_2 = L40_1
      L5_2 = L2_2
      L3_2(L4_2, L5_2)
      L3_2 = L9_1
      L3_2 = L3_2.sleep
      L4_2 = L43_1
      L3_2(L4_2)
      L3_2 = L40_1
      L4_2 = L40_1
      L4_2 = #L4_2
      L2_2 = L3_2[L4_2]
      L3_2 = false
      L39_1 = L3_2
      L3_2 = {}
      L40_1 = L3_2
    else
      L3_2 = table
      L3_2 = L3_2.insert
      L4_2 = L40_1
      L5_2 = L2_2
      L3_2(L4_2, L5_2)
      L3_2 = nil
      return L3_2
    end
  end
  L3_2 = nil
  L4_2 = nil
  L5_2 = nil
  L6_2 = MAC_ENV
  if L6_2 then
    L7_2 = L2_2
    L6_2 = L2_2.convertSpace
    L8_2 = "displayP3gamma"
    L9_2 = "ProPhotoCalc"
    L6_2, L7_2, L8_2 = L6_2(L7_2, L8_2, L9_2)
    L5_2 = L8_2
    L4_2 = L7_2
    L3_2 = L6_2
  else
    L7_2 = L2_2
    L6_2 = L2_2.convertSpace
    L8_2 = "ColorMatch"
    L9_2 = "ProPhoto"
    L6_2, L7_2, L8_2 = L6_2(L7_2, L8_2, L9_2)
    L5_2 = L8_2
    L4_2 = L7_2
    L3_2 = L6_2
  end
  L6_2 = {}
  L7_2 = {}
  L8_2 = L3_2
  L7_2[1] = L8_2
  L8_2 = {}
  L9_2 = L4_2
  L8_2[1] = L9_2
  L9_2 = {}
  L10_2 = L5_2
  L9_2[1] = L10_2
  L6_2[1] = L7_2
  L6_2[2] = L8_2
  L6_2[3] = L9_2
  L7_2 = A0_2.toneProfile
  L8_2 = L15_1
  L8_2 = L8_2.toneProfiles
  L8_2 = L8_2[L7_2]
  L9_2 = A0_2.contrast
  L9_2 = -L9_2
  L10_2 = L8_2.defaultContrast
  L9_2 = L9_2 - L10_2
  L10_2 = L18_1
  L10_2 = L10_2.contrastLayer
  L11_2 = L6_2
  L12_2 = L9_2
  L10_2 = L10_2(L11_2, L12_2)
  L11_2 = L10_2[1]
  L11_2 = L11_2[1]
  L12_2 = L10_2[2]
  L12_2 = L12_2[1]
  L13_2 = L10_2[3]
  L5_2 = L13_2[1]
  L4_2 = L12_2
  L3_2 = L11_2
  L11_2 = 1
  L12_2 = L8_2.autoTone
  if L12_2 then
    L12_2 = L18_1
    L12_2 = L12_2.autoToneGamma
    L13_2 = A1_2
    L14_2 = A0_2
    L12_2 = L12_2(L13_2, L14_2)
    L11_2 = L12_2
  end
  L12_2 = A0_2.brightness
  L12_2 = L12_2 * 0.02
  L12_2 = 1 + L12_2
  L13_2 = L8_2.defaultGamma
  L12_2 = L12_2 * L13_2
  L12_2 = L12_2 * L11_2
  L13_2 = {}
  L14_2 = {}
  L15_2 = L3_2
  L14_2[1] = L15_2
  L15_2 = {}
  L16_2 = L4_2
  L15_2[1] = L16_2
  L16_2 = {}
  L17_2 = L5_2
  L16_2[1] = L17_2
  L13_2[1] = L14_2
  L13_2[2] = L15_2
  L13_2[3] = L16_2
  L6_2 = L13_2
  L13_2 = L18_1
  L13_2 = L13_2.gammaLayer
  L14_2 = L6_2
  L15_2 = L12_2
  L16_2 = A0_2
  L13_2 = L13_2(L14_2, L15_2, L16_2)
  L14_2 = L13_2[1]
  L14_2 = L14_2[1]
  L15_2 = L13_2[2]
  L15_2 = L15_2[1]
  L16_2 = L13_2[3]
  L5_2 = L16_2[1]
  L4_2 = L15_2
  L3_2 = L14_2
  L14_2 = A0_2.wbTemp
  L15_2 = A0_2.wbTint
  L16_2 = {}
  L17_2 = 0
  L18_2 = -L15_2
  L18_2 = L18_2 / 255
  L19_2 = _G
  L19_2 = L19_2.tempMultiplier
  L18_2 = L18_2 * L19_2
  L19_2 = -L14_2
  L19_2 = L19_2 / 255
  L20_2 = _G
  L20_2 = L20_2.tempMultiplier
  L19_2 = L19_2 * L20_2
  L16_2[1] = L17_2
  L16_2[2] = L18_2
  L16_2[3] = L19_2
  L17_2 = L18_1
  L17_2 = L17_2.revertColorLinearLayer
  L18_2 = {}
  L19_2 = {}
  L20_2 = L3_2
  L19_2[1] = L20_2
  L20_2 = {}
  L21_2 = L4_2
  L20_2[1] = L21_2
  L21_2 = {}
  L22_2 = L5_2
  L21_2[1] = L22_2
  L18_2[1] = L19_2
  L18_2[2] = L20_2
  L18_2[3] = L21_2
  L19_2 = L16_2
  L17_2 = L17_2(L18_2, L19_2)
  L18_2 = L17_2[1]
  L18_2 = L18_2[1]
  L19_2 = L17_2[2]
  L19_2 = L19_2[1]
  L20_2 = L17_2[3]
  L20_2 = L20_2[1]
  if 0.95 < L18_2 or 0.95 < L19_2 or 0.95 < L20_2 then
    L21_2 = L3_1
    L21_2 = L21_2.message
    L22_2 = "The area you've selected it too bright. Please try something closer to neutral grey"
    L21_2(L22_2)
    L21_2 = nil
    return L21_2
  end
  if L18_2 < 0.05 or L19_2 < 0.05 or L20_2 < 0.05 then
    L21_2 = L3_1
    L21_2 = L21_2.message
    L22_2 = "The area you've selected it too dark. Please try something closer to neutral grey"
    L21_2(L22_2)
    L21_2 = nil
    return L21_2
  end
  L21_2 = L17_1
  L22_2 = L18_2
  L23_2 = L19_2
  L24_2 = L20_2
  L21_2 = L21_2(L22_2, L23_2, L24_2)
  L23_2 = L21_2
  L22_2 = L21_2.linearBalance
  L24_2 = "r"
  L22_2 = L22_2(L23_2, L24_2)
  L23_2 = {}
  L24_2 = 0
  L25_2 = L22_2[2]
  L25_2 = -L25_2
  L26_2 = _G
  L26_2 = L26_2.tempMultiplier
  L25_2 = L25_2 / L26_2
  L26_2 = L22_2[3]
  L26_2 = -L26_2
  L27_2 = _G
  L27_2 = L27_2.tempMultiplier
  L26_2 = L26_2 / L27_2
  L23_2[1] = L24_2
  L23_2[2] = L25_2
  L23_2[3] = L26_2
  return L23_2
end

L45_1 = NLPViewController

function L46_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = A0_2.sharpening
  if L1_2 == "none" then
    L1_2 = {}
    L2_2 = "ColorNoiseReduction"
    L3_2 = "ColorNoiseReductionDetail"
    L4_2 = "ColorNoiseReductionSmoothness"
    L5_2 = "LuminanceNoiseReductionContrast"
    L6_2 = "LuminanceNoiseReductionDetail"
    L7_2 = "LuminanceSmoothing"
    L8_2 = "SharpenDetail"
    L9_2 = "SharpenEdgeMasking"
    L10_2 = "SharpenRadius"
    L11_2 = "Sharpness"
    L1_2[1] = L2_2
    L1_2[2] = L3_2
    L1_2[3] = L4_2
    L1_2[4] = L5_2
    L1_2[5] = L6_2
    L1_2[6] = L7_2
    L1_2[7] = L8_2
    L1_2[8] = L9_2
    L1_2[9] = L10_2
    L1_2[10] = L11_2
    L2_2 = {}
    L3_2 = 1
    L4_2 = #L1_2
    L5_2 = 1
    for L6_2 = L3_2, L4_2, L5_2 do
      L7_2 = L1_2[L6_2]
      L8_2 = _G
      L8_2 = L8_2.activePhotosMeta
      L9_2 = _G
      L9_2 = L9_2.activePhotoIndex
      L8_2 = L8_2[L9_2]
      L8_2 = L8_2.previousLightroomSettings
      L9_2 = L1_2[L6_2]
      L8_2 = L8_2[L9_2]
      L2_2[L7_2] = L8_2
    end
    return L2_2
  end
  L1_2 = L14_1
  L1_2 = L1_2.copyTable
  L2_2 = L15_1
  L3_2 = A0_2.source
  L2_2 = L2_2[L3_2]
  L2_2 = L2_2.sharpening
  L3_2 = A0_2.sharpening
  L2_2 = L2_2[L3_2]
  L1_2 = L1_2(L2_2)
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L3_2 = _G
  L3_2 = L3_2.activePhotoIndex
  L2_2 = L2_2[L3_2]
  L2_2 = L2_2.megapixels
  L3_2 = 24
  L4_2 = L2_2 - L3_2
  L4_2 = L4_2 / 2
  L4_2 = L4_2 / L3_2
  L4_2 = L4_2 + 1
  L5_2 = L1_2.Sharpness
  if L5_2 ~= "" then
    L5_2 = L14_1
    L5_2 = L5_2.round
    L6_2 = L1_2.Sharpness
    L6_2 = L6_2 * L4_2
    L5_2 = L5_2(L6_2)
    if 150 < L5_2 then
      L5_2 = 150
    end
    L1_2.Sharpness = L5_2
  end
  L5_2 = L1_2.SharpenRadius
  if L5_2 ~= "" then
    L5_2 = L1_2.SharpenRadius
    L5_2 = L5_2 * L4_2
    if 3 < L5_2 then
      L5_2 = 3
    end
    if L5_2 < 1 then
      L5_2 = 1
    end
    L1_2.SharpenRadius = L5_2
  end
  L5_2 = L14_1
  L5_2 = L5_2.copyTable
  L6_2 = L1_2
  return L5_2(L6_2)
end

L45_1.updateSharpening = L46_1

function L45_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L4_2 = A0_2.rollAnalysis
  if L4_2 ~= "individual" then
    L4_2 = _G
    L4_2 = L4_2.rolls
    if L4_2 == nil then
      L4_2 = _G
      L4_2 = L4_2.activePhotosMeta
      L4_2 = L4_2[A2_2]
      L4_2 = L4_2.roll
      if L4_2 == nil then
        goto lbl_20
      end
    end
    L4_2 = A0_2.rollAnalysis
    if L4_2 ~= nil then
      L4_2 = A0_2.rollAnalysis
      if L4_2 ~= "selection" then
        goto lbl_26
      end
    end
  end
  ::lbl_20::
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A2_2]
  L4_2 = L4_2.autoWB
  L3_2 = L4_2[A1_2]
  goto lbl_62
  ::lbl_26::
  L4_2 = "rollID_"
  L5_2 = A0_2.rollAnalysis
  L4_2 = L4_2 .. L5_2
  L5_2 = _G
  L5_2 = L5_2.rolls
  L5_2 = L5_2[L4_2]
  if L5_2 == nil then
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[A2_2]
    L6_2 = L6_2.roll
    if L6_2 ~= nil then
      L6_2 = _G
      L6_2 = L6_2.activePhotosMeta
      L6_2 = L6_2[A2_2]
      L6_2 = L6_2.roll
      L6_2 = L6_2.ID
      L7_2 = A0_2.rollAnalysis
      if L6_2 == L7_2 then
        L6_2 = _G
        L6_2 = L6_2.activePhotosMeta
        L6_2 = L6_2[A2_2]
        L5_2 = L6_2.roll
      end
    end
  end
  if not L5_2 then
    L6_2 = {}
    L7_2 = 0
    L8_2 = 0
    L9_2 = 0
    L6_2[1] = L7_2
    L6_2[2] = L8_2
    L6_2[3] = L9_2
    return L6_2
  end
  L6_2 = L5_2.autoWB
  L3_2 = L6_2[A1_2]
  ::lbl_62::
  if L3_2 == nil then
    L4_2 = {}
    L5_2 = 0
    L6_2 = 0
    L7_2 = 0
    L4_2[1] = L5_2
    L4_2[2] = L6_2
    L4_2[3] = L7_2
    return L4_2
  end
  L4_2 = L14_1
  L4_2 = L4_2.formatHSV
  L5_2 = L3_2
  L4_2 = L4_2(L5_2)
  L5_2 = L4_2.h
  L6_2 = L4_2.s
  L7_2 = L4_2.b
  if L6_2 == 0 then
    L8_2 = {}
    L9_2 = 0
    L10_2 = 0
    L11_2 = 0
    L8_2[1] = L9_2
    L8_2[2] = L10_2
    L8_2[3] = L11_2
    return L8_2
  else
    L8_2 = L17_1
    L8_2 = L8_2.fromHSV
    L9_2 = L5_2
    L10_2 = L6_2
    L11_2 = L7_2
    L8_2 = L8_2(L9_2, L10_2, L11_2)
    L10_2 = L8_2
    L9_2 = L8_2.linearBalance
    L11_2 = "r"
    L9_2 = L9_2(L10_2, L11_2)
    return L9_2
  end
end

L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = A0_2.film
  if L2_2 == "custom" then
    L2_2 = nil
    return L2_2
  else
    L2_2 = A0_2.film
    if L2_2 == "autoColor" then
      L2_2 = {}
      L3_2 = A0_2.rollAnalysis
      if L3_2 ~= "individual" then
        L3_2 = A0_2.rollAnalysis
        if L3_2 ~= nil then
          L3_2 = A0_2.rollAnalysis
          if L3_2 ~= "selection" then
            L3_2 = _G
            L3_2 = L3_2.rolls
            if L3_2 ~= nil then
              goto lbl_49
            end
            L3_2 = _G
            L3_2 = L3_2.activePhotosIndex
            L3_2 = L3_2[A1_2]
            L3_2 = L3_2.roll
            if L3_2 ~= nil then
              goto lbl_49
            end
          end
        end
      end
      L3_2 = L17_1
      L4_2 = _G
      L4_2 = L4_2.activePhotosMeta
      L4_2 = L4_2[A1_2]
      L4_2 = L4_2.redChannel
      L4_2 = L4_2.meanPoint
      L5_2 = _G
      L5_2 = L5_2.activePhotosMeta
      L5_2 = L5_2[A1_2]
      L5_2 = L5_2.greenChannel
      L5_2 = L5_2.meanPoint
      L6_2 = _G
      L6_2 = L6_2.activePhotosMeta
      L6_2 = L6_2[A1_2]
      L6_2 = L6_2.blueChannel
      L6_2 = L6_2.meanPoint
      L3_2 = L3_2(L4_2, L5_2, L6_2)
      L2_2 = L3_2
      goto lbl_90
      ::lbl_49::
      L3_2 = "rollID_"
      L4_2 = A0_2.rollAnalysis
      L3_2 = L3_2 .. L4_2
      L4_2 = _G
      L4_2 = L4_2.rolls
      L4_2 = L4_2[L3_2]
      if L4_2 == nil then
        L5_2 = _G
        L5_2 = L5_2.activePhotosMeta
        L5_2 = L5_2[A1_2]
        L5_2 = L5_2.roll
        if L5_2 ~= nil then
          L5_2 = _G
          L5_2 = L5_2.activePhotosMeta
          L5_2 = L5_2[A1_2]
          L5_2 = L5_2.roll
          L5_2 = L5_2.ID
          L6_2 = A0_2.rollAnalysis
          if L5_2 == L6_2 then
            L5_2 = _G
            L5_2 = L5_2.activePhotosMeta
            L5_2 = L5_2[A1_2]
            L4_2 = L5_2.roll
          end
        end
      end
      if L4_2 then
        L5_2 = L17_1
        L6_2 = L4_2.rMean
        L7_2 = L4_2.gMean
        L8_2 = L4_2.bMean
        L5_2 = L5_2(L6_2, L7_2, L8_2)
        L2_2 = L5_2
      else
        L5_2 = L17_1
        L6_2 = 0.5
        L7_2 = 0.5
        L8_2 = 0.5
        L5_2 = L5_2(L6_2, L7_2, L8_2)
        L2_2 = L5_2
      end
      ::lbl_90::
      L4_2 = L2_2
      L3_2 = L2_2.linearBalance
      L5_2 = "r"
      L3_2 = L3_2(L4_2, L5_2)
      return L3_2
    else
      L2_2 = string
      L2_2 = L2_2.match
      L3_2 = A0_2.film
      L4_2 = "smartColor"
      L2_2 = L2_2(L3_2, L4_2)
      if L2_2 then
        L2_2 = L45_1
        L3_2 = A0_2
        L4_2 = A0_2.film
        L5_2 = A1_2
        L2_2 = L2_2(L3_2, L4_2, L5_2)
        return L2_2
      else
        L2_2 = string
        L2_2 = L2_2.match
        L3_2 = A0_2.film
        L4_2 = "mix"
        L2_2 = L2_2(L3_2, L4_2)
        if L2_2 then
          L2_2 = L45_1
          L3_2 = A0_2
          L4_2 = "smartColorNeutral"
          L5_2 = A1_2
          L2_2 = L2_2(L3_2, L4_2, L5_2)
          L3_2 = L45_1
          L4_2 = A0_2
          L5_2 = "smartColorWarming"
          L6_2 = A1_2
          L3_2 = L3_2(L4_2, L5_2, L6_2)
          L4_2 = {}
          L5_2 = 0
          L6_2 = L2_2[2]
          L7_2 = L2_2[3]
          L8_2 = L3_2[3]
          L7_2 = L7_2 + L8_2
          L7_2 = L7_2 / 2
          L4_2[1] = L5_2
          L4_2[2] = L6_2
          L4_2[3] = L7_2
          return L4_2
        else
          L2_2 = L15_1
          L2_2 = L2_2.filmCharacter
          L3_2 = A0_2.film
          L2_2 = L2_2[L3_2]
          L3_2 = {}
          L4_2 = 0
          L5_2 = L2_2.magenta
          L5_2 = -L5_2
          L6_2 = _G
          L6_2 = L6_2.tempMultiplier
          L5_2 = L5_2 * L6_2
          L6_2 = L2_2.yellow
          L6_2 = -L6_2
          L7_2 = _G
          L7_2 = L7_2.tempMultiplier
          L6_2 = L6_2 * L7_2
          L3_2[1] = L4_2
          L3_2[2] = L5_2
          L3_2[3] = L6_2
          return L3_2
        end
      end
    end
  end
end

L46_1.updateFilm = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L2_2 = {}
  L3_2 = A0_2.rollAnalysis
  if L3_2 == "individual" then
    L3_2 = {}
    L4_2 = _G
    L4_2 = L4_2.activePhotosMeta
    L4_2 = L4_2[A1_2]
    L4_2 = L4_2.redChannel
    L5_2 = _G
    L5_2 = L5_2.activePhotosMeta
    L5_2 = L5_2[A1_2]
    L5_2 = L5_2.greenChannel
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[A1_2]
    L6_2 = L6_2.blueChannel
    L3_2[1] = L4_2
    L3_2[2] = L5_2
    L3_2[3] = L6_2
    L2_2 = L3_2
  else
    L3_2 = "rollID_"
    L4_2 = A0_2.rollAnalysis
    L3_2 = L3_2 .. L4_2
    L4_2 = _G
    L4_2 = L4_2.rolls
    L4_2 = L4_2[L3_2]
    L5_2 = {}
    L6_2 = L4_2.rBlack
    L5_2.blackPointOrigin = L6_2
    L6_2 = L4_2.rWhite
    L5_2.whitePointOrigin = L6_2
    L6_2 = L4_2.rMean
    L5_2.meanPoint = L6_2
    L5_2.settingName = "ToneCurvePV2012Red"
    L6_2 = {}
    L7_2 = L4_2.gBlack
    L6_2.blackPointOrigin = L7_2
    L7_2 = L4_2.gWhite
    L6_2.whitePointOrigin = L7_2
    L7_2 = L4_2.gMean
    L6_2.meanPoint = L7_2
    L6_2.settingName = "ToneCurvePV2012Green"
    L7_2 = {}
    L8_2 = L4_2.bBlack
    L7_2.blackPointOrigin = L8_2
    L8_2 = L4_2.bWhite
    L7_2.whitePointOrigin = L8_2
    L8_2 = L4_2.bMean
    L7_2.meanPoint = L8_2
    L7_2.settingName = "ToneCurvePV2012Blue"
    L8_2 = {}
    L9_2 = L5_2
    L10_2 = L6_2
    L11_2 = L7_2
    L8_2[1] = L9_2
    L8_2[2] = L10_2
    L8_2[3] = L11_2
    L2_2 = L8_2
  end
  return L2_2
end

L46_1.getOriginsOfPhoto = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L2_2 = {}
  if A1_2 == "selection" then
    L3_2 = 1
    L4_2 = activePhotos
    L4_2 = #L4_2
    L5_2 = 1
    for L6_2 = L3_2, L4_2, L5_2 do
      L7_2 = NLPViewController
      L7_2 = L7_2.getOriginsOfPhoto
      L8_2 = L6_2
      L7_2 = L7_2(L8_2)
      L8_2 = table
      L8_2 = L8_2.insert
      L9_2 = L2_2
      L10_2 = L7_2
      L8_2(L9_2, L10_2)
      L8_2 = activePhotos
      L8_2 = L8_2[L6_2]
      L9_2 = L8_2
      L8_2 = L8_2.getRawMetadata
      L10_2 = "aperture"
      L8_2 = L8_2(L9_2, L10_2)
      L9_2 = activePhotos
      L9_2 = L9_2[L6_2]
      L10_2 = L9_2
      L9_2 = L9_2.getRawMetadata
      L11_2 = "shutterSpeed"
      L9_2 = L9_2(L10_2, L11_2)
      L10_2 = activePhotos
      L10_2 = L10_2[L6_2]
      L11_2 = L10_2
      L10_2 = L10_2.getRawMetadata
      L12_2 = "isoSpeedRating"
      L10_2 = L10_2(L11_2, L12_2)
      L11_2 = L14_1
      L11_2 = L11_2.exposureValue
      L12_2 = L9_2
      L13_2 = L8_2
      L14_2 = L10_2
      L11_2(L12_2, L13_2, L14_2)
    end
  else
    L3_2 = NLPViewController
    L3_2 = L3_2.getOriginsOfPhoto
    L4_2 = A0_2
    return L3_2(L4_2)
  end
  L3_2 = L2_2
  L4_2 = autoWBTable
  return L3_2, L4_2
end

L46_1.getOriginsOfPhotos = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2
  L2_2 = {}
  L3_2 = {}
  L4_2 = {}
  L5_2 = {}
  L6_2 = {}
  L7_2 = {}
  L8_2 = {}
  L9_2 = {}
  L10_2 = {}
  L11_2 = 1
  L12_2 = #A0_2
  L13_2 = 1
  for L14_2 = L11_2, L12_2, L13_2 do
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L2_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[1]
    L17_2 = L17_2.whitePointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L3_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[1]
    L17_2 = L17_2.blackPointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L4_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[1]
    L17_2 = L17_2.meanPoint
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L5_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[2]
    L17_2 = L17_2.whitePointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L6_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[2]
    L17_2 = L17_2.blackPointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L7_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[2]
    L17_2 = L17_2.meanPoint
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L8_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[3]
    L17_2 = L17_2.whitePointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L9_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[3]
    L17_2 = L17_2.blackPointOrigin
    L15_2(L16_2, L17_2)
    L15_2 = table
    L15_2 = L15_2.insert
    L16_2 = L10_2
    L17_2 = A0_2[L14_2]
    L17_2 = L17_2[3]
    L17_2 = L17_2.meanPoint
    L15_2(L16_2, L17_2)
  end
  L11_2 = {}
  L12_2 = {}
  L12_2.whitePointOrigin = L2_2
  L12_2.blackPointOrigin = L3_2
  L12_2.meanPoint = L4_2
  L11_2.r = L12_2
  L12_2 = {}
  L12_2.whitePointOrigin = L5_2
  L12_2.blackPointOrigin = L6_2
  L12_2.meanPoint = L7_2
  L11_2.g = L12_2
  L12_2 = {}
  L12_2.whitePointOrigin = L8_2
  L12_2.blackPointOrigin = L9_2
  L12_2.meanPoint = L10_2
  L11_2.b = L12_2
  return L11_2
end

L46_1.getOriginsByChannel = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2
  L1_2 = _G
  L1_2 = L1_2.activePhotosMeta
  L1_2 = L1_2[A0_2]
  L1_2 = L1_2.autoWB
  return L1_2
end

L46_1.getAutoWBOfPhoto = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L2_2 = {}
  if A1_2 == "selection" then
    L3_2 = 1
    L4_2 = activePhotos
    L4_2 = #L4_2
    L5_2 = 1
    for L6_2 = L3_2, L4_2, L5_2 do
      L7_2 = NLPViewController
      L7_2 = L7_2.getAutoWBOfPhoto
      L8_2 = L6_2
      L7_2 = L7_2(L8_2)
      L8_2 = table
      L8_2 = L8_2.insert
      L9_2 = L2_2
      L10_2 = L7_2
      L8_2(L9_2, L10_2)
    end
  else
    L3_2 = NLPViewController
    L3_2 = L3_2.getAutoWBOfPhoto
    L4_2 = A0_2
    return L3_2(L4_2)
  end
  return L2_2
end

L46_1.getAutoWBOfPhotos = L47_1

function L46_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L3_2 = A0_2.blackPointOrigin
  L4_2 = A0_2.whitePointOrigin
  L3_2 = L3_2 - L4_2
  L4_2 = A1_2.blackPointOrigin
  L5_2 = A1_2.whitePointOrigin
  L4_2 = L4_2 - L5_2
  L5_2 = A2_2.blackPointOrigin
  L6_2 = A2_2.whitePointOrigin
  L5_2 = L5_2 - L6_2
  L6_2 = L5_2 / L3_2
  L7_2 = L4_2 / L3_2
  L8_2 = L6_2
  L9_2 = L7_2
  return L8_2, L9_2
end

findChannelMultipliers = L46_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2
  L2_2 = {}
  L3_2 = {}
  L4_2 = {}
  L5_2 = {}
  L6_2 = {}
  L3_2.r = L6_2
  L6_2 = {}
  L3_2.g = L6_2
  L6_2 = {}
  L3_2.b = L6_2
  L6_2 = {}
  L4_2.r = L6_2
  L6_2 = {}
  L4_2.g = L6_2
  L6_2 = {}
  L4_2.b = L6_2
  L6_2 = {}
  L5_2.r = L6_2
  L6_2 = {}
  L5_2.g = L6_2
  L6_2 = {}
  L5_2.b = L6_2
  L6_2 = 1
  L7_2 = #A0_2
  L8_2 = 1
  for L9_2 = L6_2, L7_2, L8_2 do
    L10_2 = A0_2[L9_2]
    L10_2 = L10_2.smartColorNeutral
    L11_2 = A0_2[L9_2]
    L11_2 = L11_2.smartColorWarming
    L12_2 = A0_2[L9_2]
    L12_2 = L12_2.smartColorCooling
    L13_2 = L14_1
    L13_2 = L13_2.formatHSV
    L14_2 = L10_2
    L13_2 = L13_2(L14_2)
    L14_2 = L14_1
    L14_2 = L14_2.formatHSV
    L15_2 = L11_2
    L14_2 = L14_2(L15_2)
    L15_2 = L14_1
    L15_2 = L15_2.formatHSV
    L16_2 = L12_2
    L15_2 = L15_2(L16_2)
    L16_2 = L14_1
    L16_2 = L16_2.HSVToRGB
    L17_2 = L13_2
    L16_2 = L16_2(L17_2)
    L17_2 = L14_1
    L17_2 = L17_2.HSVToRGB
    L18_2 = L14_2
    L17_2 = L17_2(L18_2)
    L18_2 = L14_1
    L18_2 = L18_2.HSVToRGB
    L19_2 = L15_2
    L18_2 = L18_2(L19_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L3_2.r
    L21_2 = L16_2[1]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L3_2.g
    L21_2 = L16_2[2]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L3_2.b
    L21_2 = L16_2[3]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L4_2.r
    L21_2 = L17_2[1]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L4_2.g
    L21_2 = L17_2[2]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L4_2.b
    L21_2 = L17_2[3]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L5_2.r
    L21_2 = L18_2[1]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L5_2.g
    L21_2 = L18_2[2]
    L19_2(L20_2, L21_2)
    L19_2 = table
    L19_2 = L19_2.insert
    L20_2 = L5_2.b
    L21_2 = L18_2[3]
    L19_2(L20_2, L21_2)
  end
  L6_2 = {}
  L7_2 = {}
  L8_2 = L14_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L3_2.r
  L8_2 = L8_2(L9_2)
  L9_2 = L14_1
  L9_2 = L9_2.rgbAvg
  L10_2 = L3_2.g
  L9_2 = L9_2(L10_2)
  L10_2 = L14_1
  L10_2 = L10_2.rgbAvg
  L11_2 = L3_2.b
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2 = L10_2(L11_2)
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
  L6_2.smartColorNeutral = L7_2
  L7_2 = {}
  L8_2 = L14_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L4_2.r
  L8_2 = L8_2(L9_2)
  L9_2 = L14_1
  L9_2 = L9_2.rgbAvg
  L10_2 = L4_2.g
  L9_2 = L9_2(L10_2)
  L10_2 = L14_1
  L10_2 = L10_2.rgbAvg
  L11_2 = L4_2.b
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2 = L10_2(L11_2)
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
  L6_2.smartColorWarming = L7_2
  L7_2 = {}
  L8_2 = L14_1
  L8_2 = L8_2.rgbAvg
  L9_2 = L5_2.r
  L8_2 = L8_2(L9_2)
  L9_2 = L14_1
  L9_2 = L9_2.rgbAvg
  L10_2 = L5_2.g
  L9_2 = L9_2(L10_2)
  L10_2 = L14_1
  L10_2 = L10_2.rgbAvg
  L11_2 = L5_2.b
  L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2 = L10_2(L11_2)
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
  L6_2.smartColorCooling = L7_2
  return L6_2
end

L46_1.analyzeWBWithMethod = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2
end

L46_1.findColorMultiplierOutliers = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2, L30_2, L31_2, L32_2, L33_2, L34_2, L35_2, L36_2, L37_2, L38_2, L39_2, L40_2, L41_2, L42_2, L43_2, L44_2, L45_2, L46_2
  L3_2 = {}
  L4_2 = A1_2.r
  L4_2 = L4_2.whitePointOrigin
  L5_2 = A1_2.r
  L5_2 = L5_2.blackPointOrigin
  L6_2 = A1_2.r
  L6_2 = L6_2.meanPoint
  L7_2 = A1_2.g
  L7_2 = L7_2.whitePointOrigin
  L8_2 = A1_2.g
  L8_2 = L8_2.blackPointOrigin
  L9_2 = A1_2.g
  L9_2 = L9_2.meanPoint
  L10_2 = A1_2.b
  L10_2 = L10_2.whitePointOrigin
  L11_2 = A1_2.b
  L11_2 = L11_2.blackPointOrigin
  L12_2 = A1_2.b
  L12_2 = L12_2.meanPoint
  L13_2 = L14_1
  L13_2 = L13_2.max
  L14_2 = L5_2
  L13_2 = L13_2(L14_2)
  L14_2 = L14_1
  L14_2 = L14_2.max
  L15_2 = L8_2
  L14_2 = L14_2(L15_2)
  L15_2 = L14_1
  L15_2 = L15_2.max
  L16_2 = L11_2
  L15_2 = L15_2(L16_2)
  L16_2 = nil
  L17_2 = nil
  L18_2 = nil
  if A2_2 == "minMax" then
    L19_2 = 2
    L20_2 = L14_1
    L20_2 = L20_2.standardDeviation
    L21_2 = L4_2
    L20_2, L21_2 = L20_2(L21_2)
    L22_2 = L14_1
    L22_2 = L22_2.standardDeviation
    L23_2 = L7_2
    L22_2, L23_2 = L22_2(L23_2)
    L24_2 = L14_1
    L24_2 = L24_2.standardDeviation
    L25_2 = L10_2
    L24_2, L25_2 = L24_2(L25_2)
    L26_2 = L20_2 * L19_2
    L26_2 = L21_2 - L26_2
    L27_2 = L22_2 * L19_2
    L27_2 = L23_2 - L27_2
    L28_2 = L24_2 * L19_2
    L28_2 = L25_2 - L28_2
    L29_2 = {}
    L30_2 = {}
    L31_2 = {}
    L32_2 = 1
    L33_2 = #L4_2
    L34_2 = 1
    for L35_2 = L32_2, L33_2, L34_2 do
      L36_2 = L4_2[L35_2]
      if L26_2 > L36_2 then
        L36_2 = table
        L36_2 = L36_2.insert
        L37_2 = L29_2
        L38_2 = L35_2
        L36_2(L37_2, L38_2)
      end
      L36_2 = L7_2[L35_2]
      if L27_2 > L36_2 then
        L36_2 = table
        L36_2 = L36_2.insert
        L37_2 = L30_2
        L38_2 = L35_2
        L36_2(L37_2, L38_2)
      end
      L36_2 = L10_2[L35_2]
      if L28_2 > L36_2 then
        L36_2 = table
        L36_2 = L36_2.insert
        L37_2 = L31_2
        L38_2 = L35_2
        L36_2(L37_2, L38_2)
      end
    end
    L32_2 = #L29_2
    L33_2 = 1
    L34_2 = -1
    for L35_2 = L32_2, L33_2, L34_2 do
      L36_2 = table
      L36_2 = L36_2.remove
      L37_2 = L4_2
      L38_2 = L29_2[L35_2]
      L36_2(L37_2, L38_2)
    end
    L32_2 = #L30_2
    L33_2 = 1
    L34_2 = -1
    for L35_2 = L32_2, L33_2, L34_2 do
      L36_2 = table
      L36_2 = L36_2.remove
      L37_2 = L7_2
      L38_2 = L30_2[L35_2]
      L36_2(L37_2, L38_2)
    end
    L32_2 = #L31_2
    L33_2 = 1
    L34_2 = -1
    for L35_2 = L32_2, L33_2, L34_2 do
      L36_2 = table
      L36_2 = L36_2.remove
      L37_2 = L10_2
      L38_2 = L31_2[L35_2]
      L36_2(L37_2, L38_2)
    end
    L32_2 = {}
    L32_2.blackPointOrigin = L13_2
    L33_2 = L14_1
    L33_2 = L33_2.min
    L34_2 = L4_2
    L33_2 = L33_2(L34_2)
    L32_2.whitePointOrigin = L33_2
    L33_2 = L14_1
    L33_2 = L33_2.rgbAvgInPercent
    L34_2 = L6_2
    L33_2 = L33_2(L34_2)
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Red"
    L16_2 = L32_2
    L32_2 = {}
    L32_2.blackPointOrigin = L14_2
    L33_2 = L14_1
    L33_2 = L33_2.min
    L34_2 = L7_2
    L33_2 = L33_2(L34_2)
    L32_2.whitePointOrigin = L33_2
    L33_2 = L14_1
    L33_2 = L33_2.rgbAvgInPercent
    L34_2 = L9_2
    L33_2 = L33_2(L34_2)
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Green"
    L17_2 = L32_2
    L32_2 = {}
    L32_2.blackPointOrigin = L15_2
    L33_2 = L14_1
    L33_2 = L33_2.min
    L34_2 = L10_2
    L33_2 = L33_2(L34_2)
    L32_2.whitePointOrigin = L33_2
    L33_2 = L14_1
    L33_2 = L33_2.rgbAvgInPercent
    L34_2 = L12_2
    L33_2 = L33_2(L34_2)
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Blue"
    L18_2 = L32_2
  elseif A2_2 == "minBlackMeanWhite" then
    L19_2 = {}
    L20_2 = {}
    L21_2 = {}
    L22_2 = {}
    L23_2 = {}
    L24_2 = 1
    L25_2 = #A0_2
    L26_2 = 1
    for L27_2 = L24_2, L25_2, L26_2 do
      L28_2 = A0_2[L27_2]
      L28_2 = L28_2[1]
      L28_2 = L28_2.whitePointOrigin
      L28_2 = L13_2 - L28_2
      L28_2 = L28_2 ^ 2
      L29_2 = A0_2[L27_2]
      L29_2 = L29_2[2]
      L29_2 = L29_2.whitePointOrigin
      L29_2 = L14_2 - L29_2
      L29_2 = L29_2 ^ 2
      L30_2 = A0_2[L27_2]
      L30_2 = L30_2[3]
      L30_2 = L30_2.whitePointOrigin
      L30_2 = L15_2 - L30_2
      L30_2 = L30_2 ^ 2
      L31_2 = L29_2 / L28_2
      L32_2 = L30_2 / L28_2
      L33_2 = table
      L33_2 = L33_2.insert
      L34_2 = L19_2
      L35_2 = L28_2
      L33_2(L34_2, L35_2)
      L33_2 = table
      L33_2 = L33_2.insert
      L34_2 = L20_2
      L35_2 = L29_2
      L33_2(L34_2, L35_2)
      L33_2 = table
      L33_2 = L33_2.insert
      L34_2 = L21_2
      L35_2 = L30_2
      L33_2(L34_2, L35_2)
      L33_2 = table
      L33_2 = L33_2.insert
      L34_2 = L22_2
      L35_2 = L31_2
      L33_2(L34_2, L35_2)
      L33_2 = table
      L33_2 = L33_2.insert
      L34_2 = L23_2
      L35_2 = L32_2
      L33_2(L34_2, L35_2)
    end
    L24_2 = L14_1
    L24_2 = L24_2.standardDeviation
    L25_2 = L22_2
    L24_2, L25_2 = L24_2(L25_2)
    L26_2 = L14_1
    L26_2 = L26_2.standardDeviation
    L27_2 = L23_2
    L26_2, L27_2 = L26_2(L27_2)
    L28_2 = 2
    L29_2 = {}
    L30_2 = 1
    L31_2 = #L23_2
    L32_2 = 1
    for L33_2 = L30_2, L31_2, L32_2 do
      L34_2 = L28_2 * L24_2
      L35_2 = L28_2 * L26_2
      L36_2 = L23_2[L33_2]
      L37_2 = L27_2 + L35_2
      L36_2 = L36_2 > L37_2
      L37_2 = L22_2[L33_2]
      L38_2 = L25_2 + L34_2
      L37_2 = L37_2 > L38_2
      if L36_2 or L37_2 then
        L38_2 = table
        L38_2 = L38_2.insert
        L39_2 = L29_2
        L40_2 = L33_2
        L38_2(L39_2, L40_2)
      end
    end
    L30_2 = #L29_2
    L31_2 = 1
    L32_2 = -1
    for L33_2 = L30_2, L31_2, L32_2 do
      L34_2 = table
      L34_2 = L34_2.remove
      L35_2 = L22_2
      L36_2 = L29_2[L33_2]
      L34_2(L35_2, L36_2)
      L34_2 = table
      L34_2 = L34_2.remove
      L35_2 = L23_2
      L36_2 = L29_2[L33_2]
      L34_2(L35_2, L36_2)
    end
    L30_2 = 1
    L31_2 = L14_1
    L31_2 = L31_2.mean
    L32_2 = L23_2
    L31_2 = L31_2(L32_2)
    L32_2 = L14_1
    L32_2 = L32_2.mean
    L33_2 = L22_2
    L32_2 = L32_2(L33_2)
    L33_2 = _G
    L33_2 = L33_2.activePhotoIndex
    L33_2 = A0_2[L33_2]
    L34_2 = _G
    L34_2 = L34_2.activePhotoIndex
    L34_2 = L19_2[L34_2]
    L35_2 = L34_2 * L32_2
    L36_2 = L34_2 * L31_2
    L37_2 = _G
    L37_2 = L37_2.activePhotoIndex
    L37_2 = L4_2[L37_2]
    L38_2 = math
    L38_2 = L38_2.sqrt
    L39_2 = L35_2
    L38_2 = L38_2(L39_2)
    L38_2 = L14_2 - L38_2
    L39_2 = math
    L39_2 = L39_2.sqrt
    L40_2 = L36_2
    L39_2 = L39_2(L40_2)
    L39_2 = L15_2 - L39_2
    L40_2 = nil
    L41_2 = nil
    L42_2 = nil
    L43_2 = L33_2[2]
    L43_2 = L43_2.whitePointOrigin
    L43_2 = L38_2 - L43_2
    L44_2 = L33_2[3]
    L44_2 = L44_2.whitePointOrigin
    L44_2 = L39_2 - L44_2
    if 0 < L43_2 or 0 < L44_2 then
      if L43_2 > L44_2 then
        L45_2 = _G
        L45_2 = L45_2.activePhotoIndex
        L42_2 = L20_2[L45_2]
        L40_2 = L42_2 / L32_2
        L41_2 = L40_2 * L31_2
      else
        L45_2 = _G
        L45_2 = L45_2.activePhotoIndex
        L41_2 = L21_2[L45_2]
        L40_2 = L41_2 / L31_2
        L42_2 = L40_2 * L32_2
      end
      L45_2 = math
      L45_2 = L45_2.sqrt
      L46_2 = L40_2
      L45_2 = L45_2(L46_2)
      L37_2 = L13_2 - L45_2
      L45_2 = math
      L45_2 = L45_2.sqrt
      L46_2 = L42_2
      L45_2 = L45_2(L46_2)
      L38_2 = L14_2 - L45_2
      L45_2 = math
      L45_2 = L45_2.sqrt
      L46_2 = L41_2
      L45_2 = L45_2(L46_2)
      L39_2 = L15_2 - L45_2
    end
    L45_2 = {}
    L45_2.blackPointOrigin = L13_2
    L45_2.whitePointOrigin = L37_2
    L46_2 = L33_2[1]
    L46_2 = L46_2.meanPoint
    L45_2.meanPoint = L46_2
    L45_2.settingName = "ToneCurvePV2012Red"
    L16_2 = L45_2
    L45_2 = {}
    L45_2.blackPointOrigin = L14_2
    L45_2.whitePointOrigin = L38_2
    L46_2 = L33_2[2]
    L46_2 = L46_2.meanPoint
    L45_2.meanPoint = L46_2
    L45_2.settingName = "ToneCurvePV2012Green"
    L17_2 = L45_2
    L45_2 = {}
    L45_2.blackPointOrigin = L15_2
    L45_2.whitePointOrigin = L39_2
    L46_2 = L33_2[3]
    L46_2 = L46_2.meanPoint
    L45_2.meanPoint = L46_2
    L45_2.settingName = "ToneCurvePV2012Blue"
    L18_2 = L45_2
  elseif A2_2 == "avg" then
    L19_2 = {}
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L5_2
    L20_2 = L20_2(L21_2)
    L19_2.blackPointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L4_2
    L20_2 = L20_2(L21_2)
    L19_2.whitePointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.mean
    L21_2 = L6_2
    L20_2 = L20_2(L21_2)
    L19_2.meanPoint = L20_2
    L19_2.settingName = "ToneCurvePV2012Red"
    L16_2 = L19_2
    L19_2 = {}
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L8_2
    L20_2 = L20_2(L21_2)
    L19_2.blackPointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L7_2
    L20_2 = L20_2(L21_2)
    L19_2.whitePointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.mean
    L21_2 = L9_2
    L20_2 = L20_2(L21_2)
    L19_2.meanPoint = L20_2
    L19_2.settingName = "ToneCurvePV2012Green"
    L17_2 = L19_2
    L19_2 = {}
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L11_2
    L20_2 = L20_2(L21_2)
    L19_2.blackPointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.rgbAvg
    L21_2 = L10_2
    L20_2 = L20_2(L21_2)
    L19_2.whitePointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.mean
    L21_2 = L12_2
    L20_2 = L20_2(L21_2)
    L19_2.meanPoint = L20_2
    L19_2.settingName = "ToneCurvePV2012Blue"
    L18_2 = L19_2
  elseif A2_2 == "multipliers" then
    L19_2 = {}
    L20_2 = L14_1
    L20_2 = L20_2.max
    L21_2 = L5_2
    L20_2 = L20_2(L21_2)
    L19_2.blackPointOrigin = L20_2
    L20_2 = L14_1
    L20_2 = L20_2.mean
    L21_2 = L4_2
    L20_2 = L20_2(L21_2)
    L19_2.whitePointOrigin = L20_2
    L20_2 = {}
    L21_2 = L14_1
    L21_2 = L21_2.max
    L22_2 = L8_2
    L21_2 = L21_2(L22_2)
    L20_2.blackPointOrigin = L21_2
    L21_2 = L14_1
    L21_2 = L21_2.mean
    L22_2 = L7_2
    L21_2 = L21_2(L22_2)
    L20_2.whitePointOrigin = L21_2
    L21_2 = {}
    L22_2 = L14_1
    L22_2 = L22_2.max
    L23_2 = L11_2
    L22_2 = L22_2(L23_2)
    L21_2.blackPointOrigin = L22_2
    L22_2 = L14_1
    L22_2 = L22_2.mean
    L23_2 = L10_2
    L22_2 = L22_2(L23_2)
    L21_2.whitePointOrigin = L22_2
    L22_2 = findChannelMultipliers
    L23_2 = L19_2
    L24_2 = L20_2
    L25_2 = L21_2
    L22_2, L23_2 = L22_2(L23_2, L24_2, L25_2)
    L24_2 = _G
    L24_2 = L24_2.activePhotoIndex
    L24_2 = A0_2[L24_2]
    L25_2 = L19_2.blackPointOrigin
    L26_2 = L24_2[1]
    L26_2 = L26_2.whitePointOrigin
    L25_2 = L25_2 - L26_2
    L26_2 = L20_2.blackPointOrigin
    L27_2 = L24_2[2]
    L27_2 = L27_2.whitePointOrigin
    L26_2 = L26_2 - L27_2
    L27_2 = L21_2.blackPointOrigin
    L28_2 = L24_2[3]
    L28_2 = L28_2.whitePointOrigin
    L27_2 = L27_2 - L28_2
    L28_2 = L25_2 * L22_2
    if L27_2 < L28_2 then
      L28_2 = L25_2 * L22_2
      if L28_2 then
        goto lbl_487
      end
    end
    L28_2 = L27_2
    ::lbl_487::
    L29_2 = L25_2 * L23_2
    if L26_2 < L29_2 then
      L29_2 = L25_2 * L23_2
      if L29_2 then
        goto lbl_494
      end
    end
    L29_2 = L26_2
    ::lbl_494::
    L30_2 = L20_2.blackPointOrigin
    L30_2 = L30_2 - L29_2
    L31_2 = L21_2.blackPointOrigin
    L31_2 = L31_2 - L28_2
    L32_2 = {}
    L33_2 = L19_2.blackPointOrigin
    L32_2.blackPointOrigin = L33_2
    L33_2 = L24_2[1]
    L33_2 = L33_2.whitePointOrigin
    L32_2.whitePointOrigin = L33_2
    L33_2 = L24_2[1]
    L33_2 = L33_2.meanPoint
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Red"
    L16_2 = L32_2
    L32_2 = {}
    L33_2 = L20_2.blackPointOrigin
    L32_2.blackPointOrigin = L33_2
    L32_2.whitePointOrigin = L30_2
    L33_2 = L24_2[2]
    L33_2 = L33_2.meanPoint
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Green"
    L17_2 = L32_2
    L32_2 = {}
    L33_2 = L21_2.blackPointOrigin
    L32_2.blackPointOrigin = L33_2
    L32_2.whitePointOrigin = L31_2
    L33_2 = L24_2[3]
    L33_2 = L33_2.meanPoint
    L32_2.meanPoint = L33_2
    L32_2.settingName = "ToneCurvePV2012Blue"
    L18_2 = L32_2
  end
  L19_2 = {}
  L20_2 = L16_2
  L21_2 = L17_2
  L22_2 = L18_2
  L19_2[1] = L20_2
  L19_2[2] = L21_2
  L19_2[3] = L22_2
  return L19_2
end

L46_1.analyzeRollWithMethod = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L2_2 = {}
  L3_2 = L9_1
  L3_2 = L3_2.startAsyncTask
  
  function L4_2()
    local L0_3, L1_3
  end
  
  L3_2(L4_2)
  L3_2 = A0_2.rollAnalysis
  if L3_2 == "individual" then
    L3_2 = NLPViewController
    L3_2 = L3_2.getOriginsOfPhoto
    L4_2 = A1_2
    L3_2 = L3_2(L4_2)
    L2_2 = L3_2
  else
    L3_2 = A0_2.rollAnalysis
    if L3_2 == "selection" then
      L3_2 = L13_1
      L3_2 = L3_2.createRoll
      L4_2 = activePhotos
      L5_2 = "Test Roll"
      L6_2 = true
      L3_2(L4_2, L5_2, L6_2)
      L3_2 = NLPViewController
      L3_2 = L3_2.getOriginsOfPhotos
      L4_2 = A1_2
      L5_2 = "selection"
      L3_2 = L3_2(L4_2, L5_2)
      L4_2 = NLPViewController
      L4_2 = L4_2.getOriginsByChannel
      L5_2 = L3_2
      L4_2 = L4_2(L5_2)
      L5_2 = NLPViewController
      L5_2 = L5_2.analyzeRollWithMethod
      L6_2 = L3_2
      L7_2 = L4_2
      L8_2 = rollAnalysisMethod
      L5_2 = L5_2(L6_2, L7_2, L8_2)
      L2_2 = L5_2
    end
  end
  return L2_2
end

L46_1.updateOrigins = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2
  L2_2 = L24_1
  L3_2 = L2_2
  L2_2 = L2_2.trace
  L4_2 = "In update enhanced profile"
  L2_2(L3_2, L4_2)
  L2_2 = L19_1
  L2_2 = L2_2.useEnhancedProfiles
  L3_2 = A0_2
  L2_2(L3_2)
  L2_2 = enhancedProfilesEnabled
  if not L2_2 then
    return
  end
  L2_2 = {}
  L3_2 = nil
  L4_2 = nil
  L5_2 = nil
  L6_2 = nil
  L7_2 = nil
  L8_2 = L24_1
  L9_2 = L8_2
  L8_2 = L8_2.trace
  L10_2 = A0_2.enhancedProfile
  L8_2(L9_2, L10_2)
  L8_2 = A0_2.enhancedProfile
  if L8_2 == "None" then
    L8_2 = ""
    if L8_2 then
      goto lbl_28
      L3_2 = L8_2 or L3_2
    end
  end
  L8_2 = " - "
  L9_2 = A0_2.enhancedProfile
  L3_2 = L8_2 .. L9_2
  ::lbl_28::
  L8_2 = L24_1
  L9_2 = L8_2
  L8_2 = L8_2.trace
  L10_2 = L3_2
  L8_2(L9_2, L10_2)
  L8_2 = A0_2.postSaturation
  if L8_2 == 5 then
    L8_2 = ""
    if L8_2 then
      goto lbl_41
      L4_2 = L8_2 or L4_2
    end
  end
  L8_2 = " - Sat"
  L9_2 = A0_2.postSaturation
  L4_2 = L8_2 .. L9_2
  ::lbl_41::
  L8_2 = "Negative Lab v2.3"
  L9_2 = L3_2
  L10_2 = L4_2
  L5_2 = L8_2 .. L9_2 .. L10_2
  L8_2 = L24_1
  L9_2 = L8_2
  L8_2 = L8_2.trace
  L10_2 = L5_2
  L8_2(L9_2, L10_2)
  if L5_2 == "Negative Lab v2.3" then
    L8_2 = {}
    L9_2 = {}
    L8_2.Look = L9_2
    return L8_2
  end
  L8_2 = A0_2.enhancedProfileStrength
  L8_2 = L8_2 / 100
  L7_2 = L8_2 or L7_2
  if not L8_2 then
    L7_2 = 1
  end
  L8_2 = L15_1
  L8_2 = L8_2.enhanced_profile_tables
  L6_2 = L8_2[L5_2]
  L8_2 = L24_1
  L9_2 = L8_2
  L8_2 = L8_2.trace
  L10_2 = "Look name: "
  L11_2 = L5_2
  L12_2 = " amount: "
  L13_2 = L7_2
  L14_2 = " rgbTable: "
  L15_2 = L6_2
  L10_2 = L10_2 .. L11_2 .. L12_2 .. L13_2 .. L14_2 .. L15_2
  L8_2(L9_2, L10_2)
  L8_2 = {}
  L8_2.Amount = L7_2
  L8_2.Name = L5_2
  L9_2 = {}
  L9_2.CameraProfile = "Negative Lab v2.3"
  L9_2.RGBTable = L6_2
  L8_2.Parameters = L9_2
  L2_2.Look = L8_2
  return L2_2
end

L46_1.updateEnhancedProfile = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L2_2 = {}
  L3_2 = _G
  L3_2 = L3_2.activePhotosMeta
  L3_2 = L3_2[A1_2]
  L3_2 = L3_2.croppedDimensions
  L4_2 = _G
  L4_2 = L4_2.activePhotosMeta
  L4_2 = L4_2[A1_2]
  L4_2 = L4_2.previousLightroomSettings
  L5_2 = L4_2.orientation
  L6_2 = L4_2.CropAngle
  L7_2 = L4_2.CropBottom
  L8_2 = L4_2.CropLeft
  L9_2 = L4_2.CropRight
  L10_2 = L4_2.CropTop
  L11_2 = A0_2.previewBuffer
  if L11_2 == true then
    L11_2 = 1 - L10_2
    L12_2 = A0_2.filmBorder
    L12_2 = L12_2 / 100
    L11_2 = L11_2 * L12_2
    L11_2 = L11_2 + L10_2
    L2_2.CropTop = L11_2
    L11_2 = 1 - L8_2
    L12_2 = A0_2.filmBorder
    L12_2 = L12_2 / 100
    L11_2 = L11_2 * L12_2
    L11_2 = L11_2 + L8_2
    L2_2.CropLeft = L11_2
    L11_2 = A0_2.filmBorder
    L11_2 = L11_2 / 100
    L11_2 = 1 - L11_2
    L11_2 = L7_2 * L11_2
    L2_2.CropBottom = L11_2
    L11_2 = A0_2.filmBorder
    L11_2 = L11_2 / 100
    L11_2 = 1 - L11_2
    L11_2 = L9_2 * L11_2
    L2_2.CropRight = L11_2
  else
    L2_2.CropTop = L10_2
    L2_2.CropLeft = L8_2
    L2_2.CropBottom = L7_2
    L2_2.CropRight = L9_2
  end
  return L2_2
end

L46_1.getCropSettingsForPhoto = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = {}
  L2_2 = 1
  L3_2 = _G
  L3_2 = L3_2.activePhotos
  L3_2 = #L3_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = NLPViewController
    L6_2 = L6_2.getCropSettingsForPhoto
    L7_2 = A0_2
    L8_2 = L5_2
    L6_2 = L6_2(L7_2, L8_2)
    L7_2 = table
    L7_2 = L7_2.insert
    L8_2 = L1_2
    L9_2 = {}
    L10_2 = _G
    L10_2 = L10_2.activePhotos
    L10_2 = L10_2[L5_2]
    L9_2.photo = L10_2
    L9_2.settingsForPhoto = L6_2
    L7_2(L8_2, L9_2)
  end
  return L1_2
end

L46_1.updateCropSettings = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2
  L2_2 = "Negative Lab v2.3"
  L3_2 = "Negative Lab v2.3"
  L4_2 = A0_2.colorModel
  if L4_2 ~= "none" then
    L4_2 = A0_2.imageType
    if L4_2 ~= "editOnly" then
      goto lbl_58
    end
  end
  L4_2 = {}
  L5_2 = "CameraProfile"
  L6_2 = "ConvertToGrayscale"
  L7_2 = "ShadowTint"
  L8_2 = "RedHue"
  L9_2 = "RedSaturation"
  L10_2 = "GreenHue"
  L11_2 = "GreenSaturation"
  L12_2 = "BlueHue"
  L13_2 = "BlueSaturation"
  L14_2 = "LensProfileEnable"
  L15_2 = "SaturationAdjustmentRed"
  L16_2 = "SaturationAdjustmentOrange"
  L17_2 = "SaturationAdjustmentYellow"
  L18_2 = "SaturationAdjustmentGreen"
  L19_2 = "SaturationAdjustmentAqua"
  L20_2 = "SaturationAdjustmentBlue"
  L21_2 = "SaturationAdjustmentPurple"
  L22_2 = "SaturationAdjustmentMagenta"
  L23_2 = "Look"
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
  L4_2[16] = L20_2
  L4_2[17] = L21_2
  L4_2[18] = L22_2
  L4_2[19] = L23_2
  L5_2 = {}
  L1_2 = L5_2
  L5_2 = 1
  L6_2 = #L4_2
  L7_2 = 1
  for L8_2 = L5_2, L6_2, L7_2 do
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L10_2 = _G
    L10_2 = L10_2.activePhotoIndex
    L9_2 = L9_2[L10_2]
    L9_2 = L9_2.previousLightroomSettings
    L10_2 = L4_2[L8_2]
    L9_2 = L9_2[L10_2]
    if L9_2 ~= nil then
      L9_2 = L4_2[L8_2]
      L10_2 = _G
      L10_2 = L10_2.activePhotosMeta
      L11_2 = _G
      L11_2 = L11_2.activePhotoIndex
      L10_2 = L10_2[L11_2]
      L10_2 = L10_2.previousLightroomSettings
      L11_2 = L4_2[L8_2]
      L10_2 = L10_2[L11_2]
      L1_2[L9_2] = L10_2
    end
  end
  goto lbl_90
  ::lbl_58::
  L4_2 = L14_1
  L4_2 = L4_2.union
  L5_2 = L15_1
  L6_2 = A0_2.source
  L5_2 = L5_2[L6_2]
  L5_2 = L5_2.saturation
  L6_2 = A0_2.saturation
  L5_2 = L5_2[L6_2]
  L6_2 = L15_1
  L7_2 = A0_2.source
  L6_2 = L6_2[L7_2]
  L6_2 = L6_2.colorModels
  L7_2 = A0_2.colorModel
  L6_2 = L6_2[L7_2]
  L6_2 = L6_2.lightroomSettings
  L4_2 = L4_2(L5_2, L6_2)
  L1_2 = L4_2
  L4_2 = scanSource
  if L4_2 == "vuescanRaw" then
    L1_2.CameraProfile = L2_2
  else
    L4_2 = scanSource
    if L4_2 == "cameraScan" then
      L1_2.CameraProfile = L3_2
    end
  end
  L4_2 = enhancedProfilesEnabled
  if L4_2 == true then
    L1_2.RGBTable = ""
    L4_2 = {}
    L1_2.Look = L4_2
  end
  ::lbl_90::
  return L1_2
end

L46_1.updateRawSettings = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L1_2 = A0_2.enginePreset
  if L1_2 == "custom" then
    return
  end
  L2_2 = L15_1
  L2_2 = L2_2.engine
  L2_2 = L2_2[L1_2]
  L3_2 = NLPViewController
  L3_2 = L3_2.updateMultipleSettings
  L4_2 = L2_2
  L5_2 = A0_2
  L6_2 = true
  L3_2(L4_2, L5_2, L6_2)
end

L46_1.updateEnginePreset = L47_1
L46_1 = NLPViewController

function L47_1(A0_2, A1_2, A2_2, A3_2)
  local L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L4_2 = {}
  if A2_2 == false then
    L5_2 = _G
    L5_2 = L5_2.activePhotosMeta
    L6_2 = _G
    L6_2 = L6_2.activePhotoIndex
    L5_2 = L5_2[L6_2]
    L5_2 = L5_2.redChannel
    L5_2 = L5_2.whitePointOrigin
    if L5_2 ~= nil then
      L5_2 = {}
      L6_2 = _G
      L6_2 = L6_2.activePhotos
      L7_2 = _G
      L7_2 = L7_2.activePhotoIndex
      L6_2 = L6_2[L7_2]
      L5_2.photo = L6_2
      L6_2 = {}
      L7_2 = _G
      L7_2 = L7_2.activePhotosMeta
      L8_2 = _G
      L8_2 = L8_2.activePhotoIndex
      L7_2 = L7_2[L8_2]
      L7_2 = L7_2.redChannel
      L8_2 = _G
      L8_2 = L8_2.activePhotosMeta
      L9_2 = _G
      L9_2 = L9_2.activePhotoIndex
      L8_2 = L8_2[L9_2]
      L8_2 = L8_2.greenChannel
      L9_2 = _G
      L9_2 = L9_2.activePhotosMeta
      L10_2 = _G
      L10_2 = L10_2.activePhotoIndex
      L9_2 = L9_2[L10_2]
      L9_2 = L9_2.blueChannel
      L6_2[1] = L7_2
      L6_2[2] = L8_2
      L6_2[3] = L9_2
      L5_2.rgbInput = L6_2
      L6_2 = L14_1
      L6_2 = L6_2.copyObservableTable
      L7_2 = A3_2
      L6_2 = L6_2(L7_2)
      L5_2.properties = L6_2
      L6_2 = table
      L6_2 = L6_2.insert
      L7_2 = L4_2
      L8_2 = L5_2
      L6_2(L7_2, L8_2)
      L6_2 = NLPViewController
      L6_2 = L6_2.updateAllAdjustments
      L7_2 = L4_2
      L8_2 = A0_2
      L6_2(L7_2, L8_2)
    end
  else
    L5_2 = {}
    L6_2 = _G
    L6_2 = L6_2.activePhotos
    L7_2 = _G
    L7_2 = L7_2.activePhotoIndex
    L6_2 = L6_2[L7_2]
    L5_2.photo = L6_2
    L6_2 = {}
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L8_2 = _G
    L8_2 = L8_2.activePhotoIndex
    L7_2 = L7_2[L8_2]
    L7_2 = L7_2.redChannel
    L8_2 = _G
    L8_2 = L8_2.activePhotosMeta
    L9_2 = _G
    L9_2 = L9_2.activePhotoIndex
    L8_2 = L8_2[L9_2]
    L8_2 = L8_2.greenChannel
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L10_2 = _G
    L10_2 = L10_2.activePhotoIndex
    L9_2 = L9_2[L10_2]
    L9_2 = L9_2.blueChannel
    L6_2[1] = L7_2
    L6_2[2] = L8_2
    L6_2[3] = L9_2
    L5_2.rgbInput = L6_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L7_2 = _G
    L7_2 = L7_2.activePhotoIndex
    L6_2 = L6_2[L7_2]
    L6_2 = L6_2.settings
    L5_2.properties = L6_2
    L6_2 = table
    L6_2 = L6_2.insert
    L7_2 = L4_2
    L8_2 = L5_2
    L6_2(L7_2, L8_2)
    L6_2 = NLPViewController
    L6_2 = L6_2.updateAllAdjustments
    L7_2 = L4_2
    L8_2 = A0_2
    L6_2(L7_2, L8_2)
  end
end

L46_1.toggleBefore = L47_1
L46_1 = NLPViewController

function L47_1(A0_2)
  local L1_2, L2_2
  L1_2 = {}
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A0_2]
  L2_2 = L2_2.previousLightroomSettings
  L2_2 = L2_2.CropTop
  L1_2.CropTop = L2_2
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A0_2]
  L2_2 = L2_2.previousLightroomSettings
  L2_2 = L2_2.CropLeft
  L1_2.CropLeft = L2_2
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A0_2]
  L2_2 = L2_2.previousLightroomSettings
  L2_2 = L2_2.CropBottom
  L1_2.CropBottom = L2_2
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L2_2 = L2_2[A0_2]
  L2_2 = L2_2.previousLightroomSettings
  L2_2 = L2_2.CropRight
  L1_2.CropRight = L2_2
  return L1_2
end

L46_1.resetCropForPhoto = L47_1
L46_1 = NLPViewController

function L47_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L0_2 = {}
  L1_2 = 1
  L2_2 = _G
  L2_2 = L2_2.activePhotos
  L2_2 = #L2_2
  L3_2 = 1
  for L4_2 = L1_2, L2_2, L3_2 do
    L5_2 = {}
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L4_2]
    L6_2 = L6_2.previousLightroomSettings
    L6_2 = L6_2.CropTop
    L5_2.CropTop = L6_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L4_2]
    L6_2 = L6_2.previousLightroomSettings
    L6_2 = L6_2.CropLeft
    L5_2.CropLeft = L6_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L4_2]
    L6_2 = L6_2.previousLightroomSettings
    L6_2 = L6_2.CropBottom
    L5_2.CropBottom = L6_2
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L6_2 = L6_2[L4_2]
    L6_2 = L6_2.previousLightroomSettings
    L6_2 = L6_2.CropRight
    L5_2.CropRight = L6_2
    L6_2 = table
    L6_2 = L6_2.insert
    L7_2 = L0_2
    L8_2 = {}
    L9_2 = _G
    L9_2 = L9_2.activePhotos
    L9_2 = L9_2[L4_2]
    L8_2.photo = L9_2
    L8_2.settingsForPhoto = L5_2
    L6_2(L7_2, L8_2)
  end
  L1_2 = L24_1
  L2_2 = L1_2
  L1_2 = L1_2.trace
  L3_2 = "Resetting Crop"
  L1_2(L2_2, L3_2)
  L1_2 = NLPViewController
  L1_2 = L1_2.applyLightroomSettings
  L2_2 = nil
  L3_2 = nil
  L4_2 = true
  L5_2 = L0_2
  L1_2(L2_2, L3_2, L4_2, L5_2)
end

L46_1.resetCrop = L47_1

function L46_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L13_1
  L2_2 = L2_2.loadRollDialog
  L3_2 = A0_2
  L4_2 = A1_2
  L2_2(L3_2, L4_2)
end

loadRollDialog = L46_1

function L46_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L3_2 = A0_2
  L2_2 = A0_2.removeObserver
  L4_2 = A1_2
  L5_2 = propertiesObserverTable
  L2_2(L3_2, L4_2, L5_2)
  A0_2.selectedPreset = A1_2
  L3_2 = A0_2
  L2_2 = A0_2.addObserver
  L4_2 = A1_2
  L5_2 = propertiesObserverTable
  
  function L6_2(A0_3, A1_3, A2_3, A3_3)
    local L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
    L4_3 = NLPViewController
    L4_3 = L4_3.propertiesChanged
    L5_3 = A0_3
    L6_3 = A1_3
    L7_3 = A2_3
    L8_3 = A3_3
    L9_3 = uiSettings
    L10_3 = viewSettings
    L4_3(L5_3, L6_3, L7_3, L8_3, L9_3, L10_3)
  end
  
  L2_2(L3_2, L4_2, L5_2, L6_2)
end

function L47_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L1_2 = _G
  L1_2 = L1_2.nlpPresets
  L2_2 = lastSelectedPreset
  L1_2 = L1_2[L2_2]
  L1_2 = L1_2.presetName
  L2_2 = "Delete This Preset?"
  L3_2 = "Are you sure you want to delete the preset \""
  L4_2 = L1_2
  L5_2 = "?\" This cannot be undone."
  L3_2 = L3_2 .. L4_2 .. L5_2
  L4_2 = "YES, delete this preset"
  L5_2 = "Cancel"
  L6_2 = L3_1
  L6_2 = L6_2.confirm
  L7_2 = L2_2
  L8_2 = L3_2
  L9_2 = L4_2
  L10_2 = L5_2
  L6_2 = L6_2(L7_2, L8_2, L9_2, L10_2)
  if L6_2 == "ok" then
    L7_2 = _G
    L7_2 = L7_2.nlpPresets
    L8_2 = lastSelectedPreset
    L7_2 = L7_2[L8_2]
    L7_2 = L7_2.fileName
    L8_2 = _G
    L9_2 = _G
    L9_2 = L9_2.presetDeleteList
    if not L9_2 then
      L9_2 = {}
    end
    L8_2.presetDeleteList = L9_2
    L8_2 = table
    L8_2 = L8_2.insert
    L9_2 = _G
    L9_2 = L9_2.presetDeleteList
    L10_2 = L7_2
    L8_2(L9_2, L10_2)
    L8_2 = _G
    L8_2 = L8_2.nlpPresets
    L9_2 = lastSelectedPreset
    L8_2[L9_2] = nil
    L8_2 = createPresetTable
    L8_2()
    L8_2 = updatePresetActions
    L9_2 = "empty"
    L8_2(L9_2)
    L9_2 = A0_2
    L8_2 = A0_2.removeObserver
    L10_2 = "selectedPreset"
    L11_2 = propertiesObserverTable
    L8_2(L9_2, L10_2, L11_2)
    A0_2.selectedPreset = nil
    L9_2 = A0_2
    L8_2 = A0_2.addObserver
    L10_2 = "selectedPreset"
    L11_2 = propertiesObserverTable
    
    function L12_2(A0_3, A1_3, A2_3, A3_3)
      local L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
      L4_3 = NLPViewController
      L4_3 = L4_3.propertiesChanged
      L5_3 = A0_3
      L6_3 = A1_3
      L7_3 = A2_3
      L8_3 = A3_3
      L9_3 = uiSettings
      L10_3 = viewSettings
      L4_3(L5_3, L6_3, L7_3, L8_3, L9_3, L10_3)
    end
    
    L8_2(L9_2, L10_2, L11_2, L12_2)
  else
  end
end

function L48_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = _G
  L1_2 = L1_2.nlpPresets
  L2_2 = lastSelectedPreset
  L1_2 = L1_2[L2_2]
  L1_2 = L1_2.presetName
  L2_2 = ""
  L3_2 = isPositive
  if L3_2 == true then
    L2_2 = "This preset will automatically be applied to new positive edits"
  else
    L3_2 = isMonochrome
    if L3_2 == true then
      L2_2 = "This preset will automatically be applied to new black and white conversions"
    else
      L2_2 = "This preset will automatically be applied to new color conversions"
    end
  end
  L3_2 = "Make \""
  L4_2 = L1_2
  L5_2 = "\" Your Default?"
  L3_2 = L3_2 .. L4_2 .. L5_2
  L4_2 = "YES, make this the default preset"
  L5_2 = "Cancel"
  L6_2 = L3_1
  L6_2 = L6_2.confirm
  L7_2 = L3_2
  L8_2 = L2_2
  L9_2 = L4_2
  L10_2 = L5_2
  L6_2 = L6_2(L7_2, L8_2, L9_2, L10_2)
  if L6_2 == "ok" then
    L7_2 = _G
    L7_2 = L7_2.isPositive
    if L7_2 == true then
      L7_2 = L22_1
      L8_2 = lastSelectedPreset
      L7_2.defaultPresetPositive = L8_2
      L7_2 = L22_1
      L8_2 = L22_1
      L8_2 = L8_2.defaultPresetPositive
      L7_2.defaultPresetPositive = L8_2
    else
      L7_2 = _G
      L7_2 = L7_2.isMonochrome
      if L7_2 == true then
        L7_2 = L22_1
        L8_2 = lastSelectedPreset
        L7_2.defaultPresetMono = L8_2
        L7_2 = L22_1
        L8_2 = L22_1
        L8_2 = L8_2.defaultPresetMono
        L7_2.defaultPresetMono = L8_2
      else
        L7_2 = L22_1
        L8_2 = lastSelectedPreset
        L7_2.defaultPresetColor = L8_2
        L7_2 = L22_1
        L8_2 = L22_1
        L8_2 = L8_2.defaultPresetColor
        L7_2.defaultPresetColor = L8_2
      end
    end
    L7_2 = L46_1
    L8_2 = A0_2
    L9_2 = lastSelectedPreset
    L7_2(L8_2, L9_2)
  else
    L7_2 = L46_1
    L8_2 = A0_2
    L9_2 = lastSelectedPreset
    L7_2(L8_2, L9_2)
  end
end

L49_1 = NLPViewController

function L50_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L3_2 = lastSelectedPreset
  if not L3_2 then
    L3_2 = _G
    L3_2 = L3_2.activePhotosMeta
    L4_2 = activePhotoIndex
    L3_2 = L3_2[L4_2]
    L3_2 = L3_2.settings
    L3_2 = L3_2.selectedPreset
    lastSelectedPreset = L3_2
  end
  L3_2 = A0_2.selectedPreset
  if L3_2 == "update" then
    L4_2 = NLPViewController
    L4_2 = L4_2.loadPresetDialog
    L5_2 = "update"
    L6_2 = A0_2
    L7_2 = A1_2
    L8_2 = A2_2
    L4_2(L5_2, L6_2, L7_2, L8_2)
  elseif L3_2 == "default" then
    L4_2 = L48_1
    L5_2 = A0_2
    L4_2(L5_2)
  elseif L3_2 == "delete" then
    L4_2 = L47_1
    L5_2 = A0_2
    L4_2(L5_2)
  elseif L3_2 == "save" then
    L4_2 = NLPViewController
    L4_2 = L4_2.loadPresetDialog
    L5_2 = "save"
    L6_2 = A0_2
    L7_2 = A1_2
    L8_2 = A2_2
    L4_2(L5_2, L6_2, L7_2, L8_2)
  elseif L3_2 == "reload" then
    L4_2 = lastSelectedPreset
    A0_2.selectedPreset = L4_2
  else
    lastSelectedPreset = L3_2
    L4_2 = L26_1
    L5_2 = L3_2
    L4_2 = L4_2(L5_2)
    L5_2 = updatePresetActions
    L6_2 = "new"
    L7_2 = A0_2
    L5_2(L6_2, L7_2)
    L5_2 = NLPViewController
    L5_2 = L5_2.updateMultipleSettings
    L6_2 = L4_2
    L7_2 = A0_2
    L8_2 = true
    L5_2(L6_2, L7_2, L8_2)
  end
end

L49_1.handlePresetChange = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2, A2_2, A3_2, A4_2, A5_2)
  local L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L6_2 = L23_1
  L6_2 = L6_2.registerPropertyChange
  L7_2 = A1_2
  L8_2 = A2_2
  L9_2 = A3_2
  L6_2(L7_2, L8_2, L9_2)
  if A5_2 then
    L6_2 = A5_2.showingBefore
    if L6_2 == true then
      A5_2.showingBefore = false
    end
  end
  if A2_2 ~= "selectedPreset" and A2_2 ~= "colorModel" and A2_2 ~= "saturation" and A2_2 ~= "source" and A2_2 ~= "previewBuffer" and A2_2 ~= "filmBorder" and A2_2 ~= "convertAsRoll" then
    L6_2 = presetChanged
    if L6_2 ~= nil then
      L6_2 = presetChanged
      L7_2 = A1_2.selectedPreset
    end
    if L6_2 ~= L7_2 then
      L6_2 = A1_2.selectedPreset
      presetChanged = L6_2
      L6_2 = createPresetTable
      L7_2 = presetChanged
      L6_2(L7_2)
    end
  elseif A2_2 == "selectedPreset" then
    L6_2 = nil
    presetChanged = L6_2
    L6_2 = createPresetTable
    L6_2()
  end
  if A2_2 == "colorModel" or A2_2 == "saturation" or A2_2 == "source" then
    L6_2 = NLPViewController
    L6_2 = L6_2.updateRawSettings
    L7_2 = A1_2
    L6_2 = L6_2(L7_2)
    L7_2 = {}
    L8_2 = A1_2.colorModel
    L7_2.colorModel = L8_2
    L8_2 = A1_2.saturation
    L7_2.saturation = L8_2
    L8_2 = A1_2.source
    L7_2.source = L8_2
    L8_2 = A1_2.filmBorder
    L7_2.filmBorder = L8_2
    L8_2 = {}
    L9_2 = {}
    L10_2 = 1
    L11_2 = _G
    L11_2 = L11_2.activePhotos
    L11_2 = #L11_2
    L12_2 = 1
    for L13_2 = L10_2, L11_2, L12_2 do
      L14_2 = pairs
      L15_2 = L7_2
      L14_2, L15_2, L16_2 = L14_2(L15_2)
      for L17_2, L18_2 in L14_2, L15_2, L16_2 do
        L19_2 = _G
        L19_2 = L19_2.activePhotosMeta
        L19_2 = L19_2[L13_2]
        L19_2 = L19_2.settings
        L19_2[L17_2] = L18_2
      end
      L14_2 = _G
      L14_2 = L14_2.activePhotosMeta
      L14_2 = L14_2[L13_2]
      L14_2 = L14_2.redChannel
      L14_2 = L14_2.whitePointOrigin
      if L14_2 == nil then
        L14_2 = table
        L14_2 = L14_2.insert
        L15_2 = L8_2
        L16_2 = {}
        L17_2 = _G
        L17_2 = L17_2.activePhotos
        L17_2 = L17_2[L13_2]
        L16_2.photo = L17_2
        L16_2.settingsForPhoto = L6_2
        L14_2(L15_2, L16_2)
      else
      end
    end
    L10_2 = L24_1
    L11_2 = L10_2
    L10_2 = L10_2.trace
    L12_2 = "Raw Settings changed - apply Lightroom settings"
    L10_2(L11_2, L12_2)
    L10_2 = NLPViewController
    L10_2 = L10_2.applyLightroomSettings
    L11_2 = nil
    L12_2 = _G
    L12_2 = L12_2.activePhotoIndex
    L13_2 = true
    L14_2 = L8_2
    L10_2(L11_2, L12_2, L13_2, L14_2)
  elseif A2_2 == "imageType" then
    L6_2 = 1
    L7_2 = _G
    L7_2 = L7_2.activePhotos
    L7_2 = #L7_2
    L8_2 = 1
    for L9_2 = L6_2, L7_2, L8_2 do
      L10_2 = _G
      L10_2 = L10_2.activePhotosMeta
      L10_2 = L10_2[L9_2]
      L10_2 = L10_2.settings
      L11_2 = A1_2.imageType
      L10_2.imageType = L11_2
    end
    L6_2 = A1_2.imageType
    if L6_2 == "editOnly" then
      L6_2 = L24_1
      L7_2 = L6_2
      L6_2 = L6_2.trace
      L8_2 = "Setting imageType to editOnly"
      L6_2(L7_2, L8_2)
    end
  elseif A2_2 == "previewBuffer" or A2_2 == "filmBorder" then
    L6_2 = NLPViewController
    L6_2 = L6_2.updateCropSettings
    L7_2 = A1_2
    L6_2 = L6_2(L7_2)
    L7_2 = L24_1
    L8_2 = L7_2
    L7_2 = L7_2.trace
    L9_2 = "Crop settings changed - apply Lightroom settings"
    L7_2(L8_2, L9_2)
    L7_2 = NLPViewController
    L7_2 = L7_2.applyLightroomSettings
    L8_2 = nil
    L9_2 = nil
    L10_2 = true
    L11_2 = L6_2
    L7_2(L8_2, L9_2, L10_2, L11_2)
    L7_2 = 1
    L8_2 = _G
    L8_2 = L8_2.activePhotos
    L8_2 = #L8_2
    L9_2 = 1
    for L10_2 = L7_2, L8_2, L9_2 do
      L11_2 = _G
      L11_2 = L11_2.activePhotosMeta
      L11_2 = L11_2[L10_2]
      L11_2 = L11_2.settings
      L12_2 = A1_2.previewBuffer
      L11_2.previewBuffer = L12_2
      L11_2 = _G
      L11_2 = L11_2.activePhotosMeta
      L11_2 = L11_2[L10_2]
      L11_2 = L11_2.settings
      L12_2 = A1_2.filmBorder
      L11_2.filmBorder = L12_2
    end
  elseif A2_2 == "convertAsRoll" then
    L6_2 = _G
    L6_2 = L6_2.activePhotosMeta
    L7_2 = i
    L6_2 = L6_2[L7_2]
    L6_2 = L6_2.settings
    L7_2 = convertAsRoll
    L8_2 = A1_2.convertAsRoll
    L6_2[L7_2] = L8_2
  elseif A2_2 == "sharpening" then
    L6_2 = NLPViewController
    L6_2 = L6_2.updateSharpening
    L7_2 = A1_2
    L6_2 = L6_2(L7_2)
    if L6_2 ~= nil then
      L7_2 = L24_1
      L8_2 = L7_2
      L7_2 = L7_2.trace
      L9_2 = "Sharpening settings changed - apply Lightroom settings"
      L7_2(L8_2, L9_2)
      L7_2 = NLPViewController
      L7_2 = L7_2.applyLightroomSettings
      L8_2 = L6_2
      L9_2 = _G
      L9_2 = L9_2.activePhotoIndex
      L10_2 = true
      L7_2(L8_2, L9_2, L10_2)
    end
  elseif A2_2 == "filmHue" or A2_2 == "hueStrength" then
  elseif A2_2 == "film" then
    L6_2 = NLPViewController
    L6_2 = L6_2.updateFilm
    L7_2 = A1_2
    L8_2 = _G
    L8_2 = L8_2.activePhotoIndex
    L6_2 = L6_2(L7_2, L8_2)
    L7_2 = true
    isFilm = L7_2
    L7_2 = L6_2[3]
    L7_2 = -L7_2
    L8_2 = _G
    L8_2 = L8_2.tempMultiplier
    L7_2 = L7_2 / L8_2
    L8_2 = L6_2[2]
    L8_2 = -L8_2
    L9_2 = _G
    L9_2 = L9_2.tempMultiplier
    L8_2 = L8_2 / L9_2
    L9_2 = NLPViewController
    L9_2 = L9_2.updateMultipleSettings
    L10_2 = {}
    L10_2.wbTemp = L7_2
    L10_2.wbTint = L8_2
    L11_2 = A1_2
    L12_2 = true
    L9_2(L10_2, L11_2, L12_2)
    L9_2 = false
    isFilm = L9_2
  elseif A2_2 == "colorPicker" then
    L6_2 = L9_1
    L6_2 = L6_2.startAsyncTask
    
    function L7_2()
      local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3
      L0_3 = L44_1
      L1_3 = A1_2
      L2_3 = {}
      L3_3 = _G
      L3_3 = L3_3.activePhotosMeta
      L4_3 = _G
      L4_3 = L4_3.activePhotoIndex
      L3_3 = L3_3[L4_3]
      L3_3 = L3_3.redChannel
      L4_3 = _G
      L4_3 = L4_3.activePhotosMeta
      L5_3 = _G
      L5_3 = L5_3.activePhotoIndex
      L4_3 = L4_3[L5_3]
      L4_3 = L4_3.greenChannel
      L5_3 = _G
      L5_3 = L5_3.activePhotosMeta
      L6_3 = _G
      L6_3 = L6_3.activePhotoIndex
      L5_3 = L5_3[L6_3]
      L5_3 = L5_3.blueChannel
      L2_3[1] = L3_3
      L2_3[2] = L4_3
      L2_3[3] = L5_3
      L0_3 = L0_3(L1_3, L2_3)
      if L0_3 ~= nil then
        L1_3 = A1_2
        L2_3 = L0_3[3]
        L1_3.wbTemp = L2_3
        L1_3 = A1_2
        L2_3 = L0_3[2]
        L1_3.wbTint = L2_3
      end
    end
    
    L6_2(L7_2)
  else
    if A2_2 == "enhancedProfile" then
      L6_2 = enhancedProfilesEnabled
      if L6_2 == true then
        goto lbl_268
      end
    end
    if A2_2 == "enhancedProfileStrength" then
      L6_2 = enhancedProfilesEnabled
      if L6_2 == true then
        goto lbl_268
      end
    end
    if A2_2 == "postSaturation" then
      L6_2 = enhancedProfilesEnabled
      if L6_2 == true then
        ::lbl_268::
        L6_2 = NLPViewController
        L6_2 = L6_2.updateEnhancedProfile
        L7_2 = A1_2
        L8_2 = _G
        L8_2 = L8_2.activePhotoIndex
        L6_2 = L6_2(L7_2, L8_2)
        L7_2 = L10_1
        L7_2 = L7_2.versionTable
        L7_2 = L7_2()
        L8_2 = L7_2.major
        if not (L8_2 < 13) then
          L8_2 = L11_1
          L8_2 = L8_2.getCurrentModuleName
          L8_2 = L8_2()
          if L8_2 ~= "library" then
            goto lbl_296
          end
        end
        L8_2 = L24_1
        L9_2 = L8_2
        L8_2 = L8_2.trace
        L10_2 = "Enhanced profile settings changed - apply Lightroom settings"
        L8_2(L9_2, L10_2)
        L8_2 = NLPViewController
        L8_2 = L8_2.applyLightroomSettings
        L9_2 = L6_2
        L10_2 = activePhotoIndex
        L11_2 = true
        L8_2(L9_2, L10_2, L11_2)
        goto lbl_459
        ::lbl_296::
        L8_2 = L24_1
        L9_2 = L8_2
        L8_2 = L8_2.trace
        L10_2 = "Enhanced profile settings changed - apply Lightroom settings"
        L8_2(L9_2, L10_2)
        L8_2 = NLPViewController
        L8_2 = L8_2.applyLightroomSettings
        L9_2 = L6_2
        L10_2 = activePhotoIndex
        L11_2 = false
        L8_2(L9_2, L10_2, L11_2)
    end
    elseif A2_2 == "enginePreset" then
      L6_2 = NLPViewController
      L6_2 = L6_2.updateEnginePreset
      L7_2 = A1_2
      L6_2(L7_2)
    elseif A2_2 == "selectedPreset" then
      L6_2 = NLPViewController
      L6_2 = L6_2.handlePresetChange
      L7_2 = A1_2
      L8_2 = A4_2
      L9_2 = A5_2
      L6_2(L7_2, L8_2, L9_2)
    elseif A2_2 == "rollAnalysis" and A3_2 == "add" or A2_2 == "rollAnalysis" and A3_2 == "manage" or A2_2 == "rollAnalysis" and A3_2 == "delete" or A2_2 == "rollAnalysis" and A3_2 == "edit" then
      L6_2 = L25_1
      if not L6_2 then
        L6_2 = _G
        L6_2 = L6_2.activePhotosMeta
        L7_2 = activePhotoIndex
        L6_2 = L6_2[L7_2]
        L6_2 = L6_2.settings
        L6_2 = L6_2.rollAnalysis
        L25_1 = L6_2
      end
      L6_2 = A1_2.rollAnalysis
      if L6_2 == "add" then
        L6_2 = L13_1
        L6_2 = L6_2.loadRollDialog
        L7_2 = A1_2
        L8_2 = activePhotos
        L9_2 = L25_1
        L6_2(L7_2, L8_2, L9_2)
      else
        L6_2 = A1_2.rollAnalysis
        if L6_2 == "manage" then
          L6_2 = L13_1
          L6_2 = L6_2.loadManageRollsDialog
          L7_2 = A1_2
          L8_2 = activePhotos
          L9_2 = L25_1
          L6_2(L7_2, L8_2, L9_2)
        else
          L6_2 = A1_2.rollAnalysis
          if L6_2 == "delete" then
            L6_2 = L13_1
            L6_2 = L6_2.deleteRoll
            L7_2 = A1_2
            L8_2 = L25_1
            L6_2(L7_2, L8_2)
          else
            L6_2 = A1_2.rollAnalysis
            if L6_2 == "edit" then
              L6_2 = L13_1
              L6_2 = L6_2.editRoll
              L7_2 = A1_2
              L8_2 = L25_1
              L6_2(L7_2, L8_2)
            end
          end
        end
      end
    elseif A2_2 == "rollAnalysis" or A2_2 == "roleMode" then
      L6_2 = createRollDropdown
      L6_2()
      L6_2 = A1_2.rollAnalysis
      L25_1 = L6_2
      L6_2 = A1_2.film
      if L6_2 ~= "autoColor" then
        L6_2 = string
        L6_2 = L6_2.match
        L7_2 = A1_2.film
        L8_2 = "smartColor"
        L6_2 = L6_2(L7_2, L8_2)
        if not L6_2 then
          L6_2 = string
          L6_2 = L6_2.match
          L7_2 = A1_2.film
          L8_2 = "mix"
          L6_2 = L6_2(L7_2, L8_2)
          if not L6_2 then
            goto lbl_434
          end
        end
      end
      L6_2 = NLPViewController
      L6_2 = L6_2.updateFilm
      L7_2 = A1_2
      L8_2 = _G
      L8_2 = L8_2.activePhotoIndex
      L6_2 = L6_2(L7_2, L8_2)
      L7_2 = true
      isFilm = L7_2
      L7_2 = L6_2[3]
      L7_2 = -L7_2
      L8_2 = _G
      L8_2 = L8_2.tempMultiplier
      L7_2 = L7_2 / L8_2
      A1_2.wbTemp = L7_2
      L7_2 = L6_2[2]
      L7_2 = -L7_2
      L8_2 = _G
      L8_2 = L8_2.tempMultiplier
      L7_2 = L7_2 / L8_2
      A1_2.wbTint = L7_2
      L7_2 = false
      isFilm = L7_2
      ::lbl_434::
      L6_2 = L9_1
      L6_2 = L6_2.startAsyncTask
      
      function L7_2()
        local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
        L0_3 = L13_1
        L0_3 = L0_3.getOriginsOfPhotoForRoll
        L1_3 = A1_2
        L2_3 = activePhotoIndex
        L0_3, L1_3, L2_3, L3_3 = L0_3(L1_3, L2_3)
        L4_3 = L18_1
        L4_3 = L4_3.updateToneCurve
        L5_3 = L0_3
        L6_3 = A1_2
        L7_3 = L1_3
        L8_3 = L2_3
        L9_3 = L3_3
        L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3, L9_3)
        if L4_3 ~= nil then
          L5_3 = L24_1
          L6_3 = L5_3
          L5_3 = L5_3.trace
          L7_3 = "Roll Analysis changed - apply Lightroom settings"
          L5_3(L6_3, L7_3)
          L5_3 = NLPViewController
          L5_3 = L5_3.applyLightroomSettings
          L6_3 = L4_3
          L7_3 = activePhotoIndex
          L5_3(L6_3, L7_3)
        end
      end
      
      L6_2(L7_2)
    else
      if A2_2 == "wbTemp" or A2_2 == "wbTint" then
        L6_2 = isFilm
        if L6_2 == false then
          A1_2.film = "custom"
        end
      end
      L6_2 = L9_1
      L6_2 = L6_2.startAsyncTask
      
      function L7_2()
        local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3
        L0_3 = L13_1
        L0_3 = L0_3.getOriginsOfPhotoForRoll
        L1_3 = A1_2
        L2_3 = activePhotoIndex
        L0_3, L1_3, L2_3, L3_3 = L0_3(L1_3, L2_3)
        L4_3 = L18_1
        L4_3 = L4_3.updateToneCurve
        L5_3 = L0_3
        L6_3 = A1_2
        L7_3 = L1_3
        L8_3 = L2_3
        L9_3 = L3_3
        L4_3 = L4_3(L5_3, L6_3, L7_3, L8_3, L9_3)
        if L4_3 ~= nil then
          L5_3 = L24_1
          L6_3 = L5_3
          L5_3 = L5_3.trace
          L7_3 = "Tone Curve settings changed - apply Lightroom settings"
          L5_3(L6_3, L7_3)
          L5_3 = NLPViewController
          L5_3 = L5_3.applyLightroomSettings
          L6_3 = L4_3
          L7_3 = activePhotoIndex
          L5_3(L6_3, L7_3)
        end
      end
      
      L6_2(L7_2)
    end
  end
  ::lbl_459::
  if A2_2 ~= "selectedPreset" then
    L6_2 = updatePresetActions
    L7_2 = "update"
    L8_2 = A1_2
    L6_2(L7_2, L8_2)
  end
end

L49_1.propertiesChanged = L50_1
L49_1 = NLPViewController

function L50_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L0_2 = L19_1
  L0_2 = L0_2.isConverted
  L1_2 = activePhotoIndex
  L0_2 = L0_2(L1_2)
  if L0_2 == true then
    return
  end
  L0_2 = {}
  L1_2 = 1
  L2_2 = activePhotos
  L2_2 = #L2_2
  L3_2 = 1
  for L4_2 = L1_2, L2_2, L3_2 do
    L5_2 = L19_1
    L5_2 = L5_2.isConverted
    L6_2 = L4_2
    L5_2 = L5_2(L6_2)
    if L5_2 == false then
      L5_2 = NLPViewController
      L5_2 = L5_2.updateRawSettings
      L6_2 = activePhotosMeta
      L7_2 = activePhotoIndex
      L6_2 = L6_2[L7_2]
      L6_2 = L6_2.settings
      L5_2 = L5_2(L6_2)
      L6_2 = NLPViewController
      L6_2 = L6_2.getCropSettingsForPhoto
      L7_2 = activePhotosMeta
      L8_2 = activePhotoIndex
      L7_2 = L7_2[L8_2]
      L7_2 = L7_2.settings
      L8_2 = L4_2
      L6_2 = L6_2(L7_2, L8_2)
      L7_2 = L14_1
      L7_2 = L7_2.union
      L8_2 = L5_2
      L9_2 = L6_2
      L7_2 = L7_2(L8_2, L9_2)
      L8_2 = table
      L8_2 = L8_2.insert
      L9_2 = L0_2
      L10_2 = {}
      L11_2 = activePhotos
      L11_2 = L11_2[L4_2]
      L10_2.photo = L11_2
      L10_2.settingsForPhoto = L7_2
      L8_2(L9_2, L10_2)
    end
  end
  L1_2 = L24_1
  L2_2 = L1_2
  L1_2 = L1_2.trace
  L3_2 = "Update all unconverted - apply Lightroom settings"
  L1_2(L2_2, L3_2)
  L1_2 = NLPViewController
  L1_2 = L1_2.applyLightroomSettings
  L2_2 = nil
  L3_2 = nil
  L4_2 = true
  L5_2 = L0_2
  L1_2(L2_2, L3_2, L4_2, L5_2)
end

L49_1.updateAllUnconverted = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2, L22_2, L23_2, L24_2, L25_2, L26_2, L27_2, L28_2, L29_2
  L3_2 = {}
  L4_2 = 1
  L5_2 = #A0_2
  L6_2 = 1
  for L7_2 = L4_2, L5_2, L6_2 do
    L8_2 = A0_2[L7_2]
    L9_2 = L7_2
    L10_2 = #A0_2
    if L10_2 == 1 then
      L10_2 = _G
      L9_2 = L10_2.activePhotoIndex
    end
    L10_2 = L8_2.properties
    L11_2 = L10_2.imageType
    if L11_2 ~= "positive" then
      L11_2 = L10_2.imageType
      if L11_2 ~= "editOny" then
        L11_2 = L8_2.rgbInput
        if not L11_2 then
          goto lbl_171
        end
        L11_2 = L8_2.rgbInput
        L11_2 = L11_2[1]
        if not L11_2 then
          goto lbl_171
        end
        L11_2 = L8_2.rgbInput
        L11_2 = L11_2[1]
        L11_2 = L11_2.whitePointOrigin
        if not L11_2 then
          goto lbl_171
        end
      end
    end
    L11_2 = {}
    L12_2 = nil
    L13_2 = nil
    L14_2 = nil
    L15_2 = nil
    L16_2 = nil
    L17_2 = A0_2[L7_2]
    L17_2 = L17_2.photo
    L18_2 = L13_1
    L18_2 = L18_2.getOriginsOfPhotoForRoll
    L19_2 = L10_2
    L20_2 = L9_2
    L18_2, L19_2, L20_2, L21_2 = L18_2(L19_2, L20_2)
    L22_2 = type
    L23_2 = L10_2.colorPicker
    L22_2 = L22_2(L23_2)
    if L22_2 == "table" then
      L22_2 = L14_1
      L22_2 = L22_2.arrayToColor
      L23_2 = L10_2.colorPicker
      L22_2 = L22_2(L23_2)
      L10_2.colorPicker = L22_2
    end
    L22_2 = NLPViewController
    L22_2 = L22_2.updateSharpening
    L23_2 = L10_2
    L22_2 = L22_2(L23_2)
    L12_2 = L22_2
    if L12_2 ~= nil then
      L22_2 = L14_1
      L22_2 = L22_2.union
      L23_2 = L11_2
      L24_2 = L12_2
      L22_2 = L22_2(L23_2, L24_2)
      L11_2 = L22_2
    end
    L22_2 = NLPViewController
    L22_2 = L22_2.updateFilm
    L23_2 = L10_2
    L24_2 = L9_2
    L22_2 = L22_2(L23_2, L24_2)
    L14_2 = L22_2
    if L14_2 ~= nil then
      L22_2 = L14_2[3]
      L22_2 = -L22_2
      L23_2 = _G
      L23_2 = L23_2.tempMultiplier
      L22_2 = L22_2 / L23_2
      L10_2.wbTemp = L22_2
      L22_2 = L14_2[2]
      L22_2 = -L22_2
      L23_2 = _G
      L23_2 = L23_2.tempMultiplier
      L22_2 = L22_2 / L23_2
      L10_2.wbTint = L22_2
      if A2_2 then
        L22_2 = L14_2[3]
        L22_2 = -L22_2
        L23_2 = _G
        L23_2 = L23_2.tempMultiplier
        L22_2 = L22_2 / L23_2
        A2_2.wbTemp = L22_2
        L22_2 = L14_2[2]
        L22_2 = -L22_2
        L23_2 = _G
        L23_2 = L23_2.tempMultiplier
        L22_2 = L22_2 / L23_2
        A2_2.wbTint = L22_2
      end
    end
    L22_2 = true
    isFilm = L22_2
    L22_2 = {}
    L22_2.SplitToningShadowHue = 0
    L22_2.SplitToningShadowSaturation = 0
    L22_2.SplitToningHighlightHue = 0
    L22_2.SplitToningHighlightSaturation = 0
    L22_2.SplitToningBalance = 0
    if L22_2 ~= nil then
      L23_2 = L14_1
      L23_2 = L23_2.union
      L24_2 = L11_2
      L25_2 = L22_2
      L23_2 = L23_2(L24_2, L25_2)
      L11_2 = L23_2
    end
    L23_2 = false
    isFilm = L23_2
    L23_2 = L18_1
    L23_2 = L23_2.updateToneCurve
    L24_2 = L18_2
    L25_2 = L10_2
    L26_2 = L19_2
    L27_2 = L20_2
    L28_2 = L21_2
    L29_2 = L9_2
    L23_2 = L23_2(L24_2, L25_2, L26_2, L27_2, L28_2, L29_2)
    L15_2 = L23_2
    if L15_2 ~= nil then
      L23_2 = L14_1
      L23_2 = L23_2.union
      L24_2 = L11_2
      L25_2 = L15_2
      L23_2 = L23_2(L24_2, L25_2)
      L11_2 = L23_2
    end
    L23_2 = NLPViewController
    L23_2 = L23_2.updateEnhancedProfile
    L24_2 = L10_2
    L25_2 = L9_2
    L23_2 = L23_2(L24_2, L25_2)
    if L23_2 ~= nil then
      L24_2 = L14_1
      L24_2 = L24_2.union
      L25_2 = L11_2
      L26_2 = L23_2
      L24_2 = L24_2(L25_2, L26_2)
      L11_2 = L24_2
    end
    L24_2 = NLPViewController
    L24_2 = L24_2.resetCropForPhoto
    L25_2 = L9_2
    L24_2 = L24_2(L25_2)
    if L24_2 ~= nil then
      L25_2 = L14_1
      L25_2 = L25_2.union
      L26_2 = L11_2
      L27_2 = L24_2
      L25_2 = L25_2(L26_2, L27_2)
      L11_2 = L25_2
    end
    L25_2 = L19_1
    L25_2 = L25_2.useEnhancedProfiles
    L26_2 = L10_2
    L25_2(L26_2)
    L25_2 = table
    L25_2 = L25_2.insert
    L26_2 = L3_2
    L27_2 = {}
    L27_2.photo = L17_2
    L27_2.settingsForPhoto = L11_2
    L25_2(L26_2, L27_2)
    goto lbl_175
    ::lbl_171::
    L11_2 = L24_1
    L12_2 = L11_2
    L11_2 = L11_2.trace
    L13_2 = "Skipping photo without origins"
    L11_2(L12_2, L13_2)
    ::lbl_175::
  end
  L4_2 = L24_1
  L5_2 = L4_2
  L4_2 = L4_2.trace
  L6_2 = "Update all adjustments - apply Lightroom settings"
  L4_2(L5_2, L6_2)
  L4_2 = NLPViewController
  L4_2 = L4_2.applyLightroomSettings
  L5_2 = nil
  L6_2 = nil
  L7_2 = true
  L8_2 = L3_2
  L4_2(L5_2, L6_2, L7_2, L8_2)
end

L49_1.updateAllAdjustments = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    L9_2 = A1_2
    L8_2 = A1_2.removeObserver
    L10_2 = L6_2
    L11_2 = propertiesObserverTable
    L8_2(L9_2, L10_2, L11_2)
  end
  L3_2 = pairs
  L4_2 = A0_2
  L3_2, L4_2, L5_2 = L3_2(L4_2)
  for L6_2, L7_2 in L3_2, L4_2, L5_2 do
    A1_2[L6_2] = L7_2
  end
  L3_2 = L14_1
  L3_2 = L3_2.copyObservableTable
  L4_2 = A1_2
  L3_2 = L3_2(L4_2)
  L4_2 = {}
  L5_2 = _G
  L5_2 = L5_2.activePhotos
  L6_2 = _G
  L6_2 = L6_2.activePhotoIndex
  L5_2 = L5_2[L6_2]
  L4_2.photo = L5_2
  L5_2 = {}
  L6_2 = _G
  L6_2 = L6_2.activePhotosMeta
  L7_2 = _G
  L7_2 = L7_2.activePhotoIndex
  L6_2 = L6_2[L7_2]
  L6_2 = L6_2.redChannel
  L7_2 = _G
  L7_2 = L7_2.activePhotosMeta
  L8_2 = _G
  L8_2 = L8_2.activePhotoIndex
  L7_2 = L7_2[L8_2]
  L7_2 = L7_2.greenChannel
  L8_2 = _G
  L8_2 = L8_2.activePhotosMeta
  L9_2 = _G
  L9_2 = L9_2.activePhotoIndex
  L8_2 = L8_2[L9_2]
  L8_2 = L8_2.blueChannel
  L5_2[1] = L6_2
  L5_2[2] = L7_2
  L5_2[3] = L8_2
  L4_2.rgbInput = L5_2
  L4_2.properties = L3_2
  L5_2 = NLPViewController
  L5_2 = L5_2.updateAllAdjustments
  L6_2 = {}
  L7_2 = L4_2
  L6_2[1] = L7_2
  L7_2 = uiSettings
  L8_2 = A1_2
  L5_2(L6_2, L7_2, L8_2)
  if A2_2 == true then
    L5_2 = L23_1
    L5_2 = L5_2.registerMultipleChanges
    L6_2 = L3_2
    L5_2(L6_2)
  end
  L5_2 = pairs
  L6_2 = A0_2
  L5_2, L6_2, L7_2 = L5_2(L6_2)
  for L8_2, L9_2 in L5_2, L6_2, L7_2 do
    L11_2 = A1_2
    L10_2 = A1_2.addObserver
    L12_2 = L8_2
    L13_2 = propertiesObserverTable
    
    function L14_2(A0_3, A1_3, A2_3, A3_3)
      local L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3
      L4_3 = NLPViewController
      L4_3 = L4_3.propertiesChanged
      L5_3 = A0_3
      L6_3 = A1_3
      L7_3 = A2_3
      L8_3 = A3_3
      L9_3 = uiSettings
      L10_3 = viewSettings
      L4_3(L5_3, L6_3, L7_3, L8_3, L9_3, L10_3)
    end
    
    L10_2(L11_2, L12_2, L13_2, L14_2)
  end
end

L49_1.updateMultipleSettings = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2
  L2_2 = L2_1
  L2_2 = L2_2.postAsyncTaskWithContext
  L3_2 = "sync photo settings"
  
  function L4_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3
    L1_3 = {}
    L2_3 = L5_1
    L3_3 = {}
    L3_3.title = "Syncing Settings..."
    L3_3.functionContext = A0_3
    L2_3 = L2_3(L3_3)
    L3_3 = 1
    L4_3 = _G
    L4_3 = L4_3.activePhotos
    L4_3 = #L4_3
    L5_3 = 1
    for L6_3 = L3_3, L4_3, L5_3 do
      L8_3 = L2_3
      L7_3 = L2_3.setPortionComplete
      L9_3 = _G
      L9_3 = L9_3.activePhotos
      L9_3 = #L9_3
      L9_3 = L6_3 / L9_3
      L9_3 = L9_3 - 0.1
      L7_3(L8_3, L9_3)
      L7_3 = pairs
      L8_3 = A0_2
      L7_3, L8_3, L9_3 = L7_3(L8_3)
      for L10_3, L11_3 in L7_3, L8_3, L9_3 do
        L12_3 = _G
        L12_3 = L12_3.activePhotosMeta
        L12_3 = L12_3[L6_3]
        L12_3 = L12_3.settings
        L12_3[L10_3] = L11_3
      end
      L7_3 = _G
      L7_3 = L7_3.activePhotos
      L7_3 = L7_3[L6_3]
      L8_3 = {}
      L9_3 = _G
      L9_3 = L9_3.activePhotosMeta
      L9_3 = L9_3[L6_3]
      L9_3 = L9_3.redChannel
      L10_3 = _G
      L10_3 = L10_3.activePhotosMeta
      L10_3 = L10_3[L6_3]
      L10_3 = L10_3.greenChannel
      L11_3 = _G
      L11_3 = L11_3.activePhotosMeta
      L11_3 = L11_3[L6_3]
      L11_3 = L11_3.blueChannel
      L8_3[1] = L9_3
      L8_3[2] = L10_3
      L8_3[3] = L11_3
      L9_3 = table
      L9_3 = L9_3.insert
      L10_3 = L1_3
      L11_3 = {}
      L11_3.photo = L7_3
      L11_3.rgbInput = L8_3
      L12_3 = _G
      L12_3 = L12_3.activePhotosMeta
      L12_3 = L12_3[L6_3]
      L12_3 = L12_3.settings
      L11_3.properties = L12_3
      L9_3(L10_3, L11_3)
    end
    L4_3 = L2_3
    L3_3 = L2_3.setPortionComplete
    L5_3 = 1
    L3_3(L4_3, L5_3)
    L4_3 = L2_3
    L3_3 = L2_3.done
    L3_3(L4_3)
    L3_3 = NLPViewController
    L3_3 = L3_3.updateAllAdjustments
    L4_3 = L1_3
    L5_3 = A1_2
    L3_3(L4_3, L5_3)
  end
  
  L2_2(L3_2, L4_2)
end

L49_1.syncPhotoSettings = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2)
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

L49_1.removeMaskCurve = L50_1
L49_1 = NLPViewController

function L50_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L0_2 = {}
  L1_2 = 1
  L2_2 = activePhotos
  L2_2 = #L2_2
  L3_2 = 1
  for L4_2 = L1_2, L2_2, L3_2 do
    L5_2 = activePhotosMeta
    L5_2 = L5_2[L4_2]
    L5_2 = L5_2.previousLightroomSettings
    L6_2 = activePhotos
    L6_2 = L6_2[L4_2]
    L7_2 = table
    L7_2 = L7_2.insert
    L8_2 = L0_2
    L9_2 = {}
    L9_2.photo = L6_2
    L9_2.settingsForPhoto = L5_2
    L7_2(L8_2, L9_2)
  end
  L1_2 = NLPViewController
  L1_2 = L1_2.applyLightroomSettings
  L2_2 = nil
  L3_2 = nil
  L4_2 = true
  L5_2 = L0_2
  L1_2(L2_2, L3_2, L4_2, L5_2)
end

L49_1.restorePreviousSettings = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2
  L2_2 = {}
  L3_2 = 1
  L4_2 = _G
  L4_2 = L4_2.activePhotos
  L4_2 = #L4_2
  L5_2 = 1
  for L6_2 = L3_2, L4_2, L5_2 do
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.redChannel
    L7_2.whitePointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.redChannel
    L7_2.blackPointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.redChannel
    L7_2.meanPoint = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.greenChannel
    L7_2.whitePointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.greenChannel
    L7_2.blackPointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.greenChannel
    L7_2.meanPoint = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.blueChannel
    L7_2.whitePointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.blueChannel
    L7_2.blackPointOrigin = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2 = L7_2.blueChannel
    L7_2.meanPoint = nil
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L8_2 = _G
    L8_2 = L8_2.version
    L7_2.nlpVersion = L8_2
    L7_2 = _G
    L7_2 = L7_2.activePhotosMeta
    L7_2 = L7_2[L6_2]
    L7_2.autoWB = nil
    if A1_2 == true then
      L7_2 = L24_1
      L8_2 = L7_2
      L7_2 = L7_2.trace
      L9_2 = "HARD RESET"
      L7_2(L8_2, L9_2)
      L7_2 = pairs
      L8_2 = _G
      L8_2 = L8_2.activePhotosMeta
      L8_2 = L8_2[L6_2]
      L8_2 = L8_2.settings
      L7_2, L8_2, L9_2 = L7_2(L8_2)
      for L10_2, L11_2 in L7_2, L8_2, L9_2 do
        if L10_2 ~= "colorModel" and L10_2 ~= "source" and L10_2 ~= "filmBorder" and L10_2 ~= "previewBuffer" and L10_2 ~= "imageType" and L10_2 ~= "saturation" then
          L12_2 = _G
          L12_2 = L12_2.activePhotosMeta
          L12_2 = L12_2[L6_2]
          L12_2 = L12_2.settings
          L12_2[L10_2] = nil
        end
      end
    end
    L7_2 = {}
    L8_2 = {}
    L9_2 = 0
    L10_2 = 0
    L11_2 = 255
    L12_2 = 255
    L8_2[1] = L9_2
    L8_2[2] = L10_2
    L8_2[3] = L11_2
    L8_2[4] = L12_2
    L7_2.ToneCurvePV2012Red = L8_2
    L8_2 = {}
    L9_2 = 0
    L10_2 = 0
    L11_2 = 255
    L12_2 = 255
    L8_2[1] = L9_2
    L8_2[2] = L10_2
    L8_2[3] = L11_2
    L8_2[4] = L12_2
    L7_2.ToneCurvePV2012Blue = L8_2
    L8_2 = {}
    L9_2 = 0
    L10_2 = 0
    L11_2 = 255
    L12_2 = 255
    L8_2[1] = L9_2
    L8_2[2] = L10_2
    L8_2[3] = L11_2
    L8_2[4] = L12_2
    L7_2.ToneCurvePV2012Green = L8_2
    L7_2.SplitToningShadowHue = 0
    L7_2.SplitToningShadowSaturation = 0
    L7_2.SplitToningBalance = 0
    L7_2.SplitToningHighlightHue = 0
    L7_2.SplitToningHighlightSaturation = 0
    L8_2 = NLPViewController
    L8_2 = L8_2.updateRawSettings
    L9_2 = _G
    L9_2 = L9_2.activePhotosMeta
    L9_2 = L9_2[L6_2]
    L9_2 = L9_2.settings
    L8_2 = L8_2(L9_2)
    if L8_2 ~= nil then
      L9_2 = pairs
      L10_2 = L8_2
      L9_2, L10_2, L11_2 = L9_2(L10_2)
      for L12_2, L13_2 in L9_2, L10_2, L11_2 do
        L7_2[L12_2] = L13_2
      end
    end
    L9_2 = NLPViewController
    L9_2 = L9_2.getCropSettingsForPhoto
    L10_2 = _G
    L10_2 = L10_2.activePhotosMeta
    L11_2 = activePhotoIndex
    L10_2 = L10_2[L11_2]
    L10_2 = L10_2.settings
    L11_2 = L6_2
    L9_2 = L9_2(L10_2, L11_2)
    L10_2 = L14_1
    L10_2 = L10_2.union
    L11_2 = L7_2
    L12_2 = L9_2
    L10_2 = L10_2(L11_2, L12_2)
    L7_2 = L10_2
    L10_2 = NLPViewController
    L10_2 = L10_2.removeMaskCurve
    L11_2 = L7_2
    L12_2 = L6_2
    L10_2 = L10_2(L11_2, L12_2)
    L7_2 = L10_2
    L10_2 = {}
    L11_2 = _G
    L11_2 = L11_2.activePhotos
    L11_2 = L11_2[L6_2]
    L10_2.photo = L11_2
    L10_2.settingsForPhoto = L7_2
    L11_2 = table
    L11_2 = L11_2.insert
    L12_2 = L2_2
    L13_2 = L10_2
    L11_2(L12_2, L13_2)
  end
  L3_2 = L24_1
  L4_2 = L3_2
  L3_2 = L3_2.trace
  L5_2 = "Reset photos - apply Lightroom settings"
  L3_2(L4_2, L5_2)
  L3_2 = NLPViewController
  L3_2 = L3_2.applyLightroomSettings
  L4_2 = resetLightroomSettings
  L5_2 = i
  L6_2 = true
  L7_2 = L2_2
  L3_2(L4_2, L5_2, L6_2, L7_2)
  L3_2 = L9_1
  L3_2 = L3_2.startAsyncTask
  
  function L4_2()
    local L0_3, L1_3
    L0_3 = L19_1
    L0_3 = L0_3.saveMetadataForAllPhotos
    L1_3 = "settings"
    L0_3(L1_3)
  end
  
  L3_2(L4_2)
  L3_2 = ""
  L4_2 = _G
  L4_2 = L4_2.activePhotos
  L4_2 = #L4_2
  if 1 < L4_2 then
    L4_2 = "CONVERT "
    L5_2 = _G
    L5_2 = L5_2.activePhotos
    L5_2 = #L5_2
    L6_2 = " NEGATIVES"
    L3_2 = L4_2 .. L5_2 .. L6_2
  else
    L3_2 = "CONVERT 1 NEGATIVE"
  end
  A0_2.convertButtonIsEnabled = true
  A0_2.colorModelIsEnabled = true
  A0_2.convertButtonTitle = L3_2
  A0_2.unconvertIsVisible = false
end

L49_1.resetPhotos = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2
  L2_2 = L24_1
  L3_2 = L2_2
  L2_2 = L2_2.trace
  L4_2 = "IN MAKE POSITIVE COPY"
  L2_2(L3_2, L4_2)
  L2_2 = ipairs
  L3_2 = A0_2
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    L7_2 = L24_1
    L8_2 = L7_2
    L7_2 = L7_2.info
    L9_2 = "Photo "
    L10_2 = L5_2
    L11_2 = " type: "
    L12_2 = type
    L13_2 = L6_2
    L12_2 = L12_2(L13_2)
    L13_2 = ", tostring: "
    L14_2 = tostring
    L15_2 = L6_2
    L14_2 = L14_2(L15_2)
    L9_2 = L9_2 .. L10_2 .. L11_2 .. L12_2 .. L13_2 .. L14_2
    L7_2(L8_2, L9_2)
    if L6_2 then
      L7_2 = type
      L8_2 = L6_2
      L7_2 = L7_2(L8_2)
      if L7_2 == "userdata" then
        L7_2 = pcall
        
        function L8_2()
          local L0_3, L1_3, L2_3
          L0_3 = L6_2
          L1_3 = L0_3
          L0_3 = L0_3.getRawMetadata
          L2_3 = "path"
          return L0_3(L1_3, L2_3)
        end
        
        L7_2, L8_2 = L7_2(L8_2)
        L9_2 = L24_1
        L10_2 = L9_2
        L9_2 = L9_2.info
        L11_2 = "Photo "
        L12_2 = L5_2
        L13_2 = " path: "
        L14_2 = L8_2 or L14_2
        if not L7_2 or not L8_2 then
          L14_2 = "Failed to get path"
        end
        L11_2 = L11_2 .. L12_2 .. L13_2 .. L14_2
        L9_2(L10_2, L11_2)
      end
    end
  end
  L2_2 = L9_1
  L2_2 = L2_2.startAsyncTask
  
  function L3_2()
    local L0_3, L1_3, L2_3
    L0_3 = L2_1
    L0_3 = L0_3.callWithContext
    L1_3 = "export"
    
    function L2_3(A0_4)
      local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4, L15_4, L16_4, L17_4, L18_4, L19_4, L20_4, L21_4, L22_4, L23_4, L24_4
      L1_4 = L5_1
      L2_4 = {}
      L3_4 = A0_2
      L3_4 = #L3_4
      if 1 < L3_4 then
        L3_4 = "Processing "
        L4_4 = A0_2
        L4_4 = #L4_4
        L5_4 = " photos"
        L3_4 = L3_4 .. L4_4 .. L5_4
        if L3_4 then
          goto lbl_15
        end
      end
      L3_4 = "Processing one photo"
      ::lbl_15::
      L2_4.title = L3_4
      L2_4.functionContext = A0_4
      L1_4 = L1_4(L2_4)
      L2_4 = {}
      L3_4 = A1_2
      L3_4 = L3_4.fileType
      if L3_4 == "tif" then
        L2_4.LR_format = "TIFF"
        L3_4 = A1_2
        L3_4 = L3_4.exportColorspace
        if not L3_4 then
          L3_4 = "sRGB"
        end
        L2_4.LR_export_colorSpace = L3_4
        L3_4 = A1_2
        L3_4 = L3_4.tifBits
        if not L3_4 then
          L3_4 = "16"
        end
        L2_4.LR_export_bitDepth = L3_4
        L3_4 = A1_2
        L3_4 = L3_4.tifCompress
        if not L3_4 then
          L3_4 = "compressionMethod_ZIP"
        end
        L2_4.LRtiff_compressionMethod = L3_4
      else
        L3_4 = A1_2
        L3_4 = L3_4.fileType
        if L3_4 == "jpg" then
          L2_4.LR_format = "JPEG"
          L3_4 = A1_2
          L3_4 = L3_4.exportColorspace
          if not L3_4 then
            L3_4 = "sRGB"
          end
          L2_4.LR_export_colorSpace = L3_4
          L3_4 = A1_2
          L3_4 = L3_4.jpgQuality
          L3_4 = L3_4 / 100
          if not L3_4 then
            L3_4 = 1
          end
          L2_4.LR_jpeg_quality = L3_4
        end
      end
      L3_4 = A1_2
      L3_4 = L3_4.outputType
      if L3_4 ~= "stack" then
        L3_4 = A1_2
        L3_4 = L3_4.outputType
        if L3_4 ~= "stackAbove" then
          goto lbl_72
        end
      end
      L2_4.LR_export_useSubfolder = false
      L2_4.LR_reimport_stackWithOriginal = true
      goto lbl_84
      ::lbl_72::
      L3_4 = A1_2
      L3_4 = L3_4.outputType
      if L3_4 == "subfolder" then
        L2_4.LR_export_useSubfolder = true
        L3_4 = A1_2
        L3_4 = L3_4.exportSubfolderPath
        if not L3_4 then
          L3_4 = "positives"
        end
        L2_4.LR_export_destinationPathSuffix = L3_4
        L2_4.LR_reimport_stackWithOriginal = false
      end
      ::lbl_84::
      L3_4 = A1_2
      L3_4 = L3_4.overwriteOldCopies
      if L3_4 == true then
        L2_4.LR_collisionHandling = "overwrite"
      else
        L2_4.LR_collisionHandling = "rename"
      end
      L3_4 = nil
      L4_4 = {}
      L4_4.LR_export_destinationType = "sourceFolder"
      L5_4 = L2_4.LR_export_useSubfolder
      L4_4.LR_export_useSubfolder = L5_4
      L5_4 = L2_4.LR_export_destinationPathSuffix
      L4_4.LR_export_destinationPathSuffix = L5_4
      L5_4 = L2_4.LR_reimportExportedPhoto
      L4_4.LR_reimportExportedPhoto = L5_4
      L5_4 = L2_4.LR_reimport_stackWithOriginal
      L4_4.LR_reimport_stackWithOriginal = L5_4
      L5_4 = L2_4.LR_collisionHandling
      L4_4.LR_collisionHandling = L5_4
      L4_4.LR_renamingTokensOn = true
      L4_4.LR_tokenCustomString = "positive"
      L4_4.LR_tokens = "{{image_name}}-{{custom_token}}"
      L5_4 = L2_4.LR_format
      L4_4.LR_format = L5_4
      L5_4 = L2_4.LR_export_colorSpace
      L4_4.LR_export_colorSpace = L5_4
      L5_4 = L2_4.LR_export_bitDepth
      L4_4.LR_export_bitDepth = L5_4
      L5_4 = L2_4.LRtiff_compressionMethod
      L4_4.LR_tiff_compressionMethod = L5_4
      L5_4 = L2_4.LR_jpeg_quality
      L4_4.LR_jpeg_quality = L5_4
      L4_4.LR_size_doConstrain = false
      L4_4.LR_size_doNotEnlarge = true
      L4_4.LR_useWatermark = false
      L4_4.LR_outputSharpeningOn = false
      L4_4.LR_embeddedMetadataOption = "all"
      L4_4.LR_removeLocationMetadata = false
      L3_4 = L4_4
      L4_4 = {}
      L5_4 = "rgbOrigins"
      L6_4 = "settings"
      L7_4 = "autoWB"
      L8_4 = "roll"
      L9_4 = "rollName"
      L10_4 = "rollID"
      L11_4 = "nlpConverted"
      L4_4[1] = L5_4
      L4_4[2] = L6_4
      L4_4[3] = L7_4
      L4_4[4] = L8_4
      L4_4[5] = L9_4
      L4_4[6] = L10_4
      L4_4[7] = L11_4
      L5_4 = {}
      L6_4 = ipairs
      L7_4 = L4_4
      L6_4, L7_4, L8_4 = L6_4(L7_4)
      for L9_4, L10_4 in L6_4, L7_4, L8_4 do
        L5_4[L10_4] = true
      end
      L6_4 = {}
      L7_4 = 1
      L8_4 = A0_2
      L8_4 = #L8_4
      L9_4 = 1
      for L10_4 = L7_4, L8_4, L9_4 do
        L11_4 = {}
        L12_4 = 1
        L13_4 = L21_1
        L13_4 = #L13_4
        L14_4 = 1
        for L15_4 = L12_4, L13_4, L14_4 do
          L16_4 = L21_1
          L16_4 = L16_4[L15_4]
          L16_4 = L16_4.id
          L17_4 = L5_4[L16_4]
          if not L17_4 then
            L17_4 = A0_2
            L17_4 = L17_4[L10_4]
            L18_4 = L17_4
            L17_4 = L17_4.getPropertyForPlugin
            L19_4 = _PLUGIN
            L20_4 = L16_4
            L17_4 = L17_4(L18_4, L19_4, L20_4)
            L11_4[L16_4] = L17_4
          end
        end
        L12_4 = table
        L12_4 = L12_4.insert
        L13_4 = L6_4
        L14_4 = L11_4
        L12_4(L13_4, L14_4)
      end
      L7_4 = {}
      L7_4.writeToExif = true
      L7_4.generateKeywords = true
      L7_4.reimportWithMetadata = true
      L7_4.includeEquipmentCaption = true
      L7_4.includeScanningCaption = true
      L7_4.includeDevelopmentCaption = true
      L7_4.appendCaption = true
      L7_4.copyCaption = true
      L7_4.captionSeparator = " | "
      L8_4 = L6_1
      L9_4 = {}
      L10_4 = A0_2
      L9_4.photosToExport = L10_4
      L9_4.exportSettings = L3_4
      L8_4 = L8_4(L9_4)
      exportSession = L8_4
      L8_4 = {}
      L8_4.progressScope = L1_4
      L8_4.renderProgressPortion = 1
      L8_4.stopIfCanceled = true
      L9_4 = {}
      L10_4 = exportSession
      L11_4 = L10_4
      L10_4 = L10_4.renditions
      L12_4 = L8_4
      L10_4, L11_4, L12_4 = L10_4(L11_4, L12_4)
      for L13_4, L14_4 in L10_4, L11_4, L12_4 do
        L16_4 = L14_4
        L15_4 = L14_4.waitForRender
        L15_4, L16_4 = L15_4(L16_4)
        L18_4 = L1_4
        L17_4 = L1_4.isCanceled
        L17_4 = L17_4(L18_4)
        if L17_4 then
          break
        end
        if L15_4 then
          L17_4 = A1_2
          L17_4 = L17_4.excludeMetadata
          if L17_4 ~= true then
            L17_4 = MAC_ENV
            if L17_4 then
              L17_4 = " -m -overwrite_original -E "
              if L17_4 then
                goto lbl_220
              end
            end
            L17_4 = " -charset Latin -m -overwrite_original -E "
            ::lbl_220::
            L18_4 = Exif
            L18_4 = L18_4.generateCommands
            L19_4 = A0_2
            L19_4 = L19_4[L13_4]
            L20_4 = L6_4[L13_4]
            L21_4 = L7_4
            L22_4 = L13_4
            L18_4 = L18_4(L19_4, L20_4, L21_4, L22_4)
            L19_4 = L14_4.destinationPath
            L20_4 = Exif
            L20_4 = L20_4.execute
            L21_4 = L17_4
            L22_4 = L18_4
            L23_4 = {}
            L24_4 = L19_4
            L23_4[1] = L24_4
            L20_4(L21_4, L22_4, L23_4)
          end
          L17_4 = L12_1
          L18_4 = L17_4
          L17_4 = L17_4.findPhotoByPath
          L19_4 = L14_4.destinationPath
          L17_4 = L17_4(L18_4, L19_4)
          if L17_4 == nil then
            L18_4 = L12_1
            L19_4 = L18_4
            L18_4 = L18_4.withWriteAccessDo
            L20_4 = "reimport photo"
            
            function L21_4(A0_5)
              local L1_5, L2_5, L3_5, L4_5, L5_5, L6_5, L7_5, L8_5, L9_5, L10_5, L11_5, L12_5
              L2_5 = A1_2
              L2_5 = L2_5.outputType
              if L2_5 == "stack" then
                L1_5 = "below"
              else
                L2_5 = A1_2
                L2_5 = L2_5.outputType
                if L2_5 == "stackAbove" then
                  L1_5 = "above"
                end
              end
              L2_5 = L12_1
              L3_5 = L2_5
              L2_5 = L2_5.addPhoto
              L4_5 = L14_4
              L4_5 = L4_5.destinationPath
              L5_5 = L14_4
              L5_5 = L5_5.photo
              L6_5 = L1_5
              L2_5 = L2_5(L3_5, L4_5, L5_5, L6_5)
              L3_5 = pairs
              L4_5 = L6_4
              L5_5 = L13_4
              L4_5 = L4_5[L5_5]
              L3_5, L4_5, L5_5 = L3_5(L4_5)
              for L6_5, L7_5 in L3_5, L4_5, L5_5 do
                if L7_5 then
                  L9_5 = L2_5
                  L8_5 = L2_5.setPropertyForPlugin
                  L10_5 = _PLUGIN
                  L11_5 = L6_5
                  L12_5 = L7_5
                  L8_5(L9_5, L10_5, L11_5, L12_5)
                end
              end
              L4_5 = L2_5
              L3_5 = L2_5.setPropertyForPlugin
              L5_5 = _PLUGIN
              L6_5 = "rgbOrigins"
              L7_5 = nil
              L3_5(L4_5, L5_5, L6_5, L7_5)
              L4_5 = L2_5
              L3_5 = L2_5.setPropertyForPlugin
              L5_5 = _PLUGIN
              L6_5 = "autoWB"
              L7_5 = nil
              L3_5(L4_5, L5_5, L6_5, L7_5)
              L4_5 = L2_5
              L3_5 = L2_5.setPropertyForPlugin
              L5_5 = _PLUGIN
              L6_5 = "settings"
              L7_5 = nil
              L3_5(L4_5, L5_5, L6_5, L7_5)
              L4_5 = L2_5
              L3_5 = L2_5.setPropertyForPlugin
              L5_5 = _PLUGIN
              L6_5 = "nlpConverted"
              L7_5 = "copy"
              L3_5(L4_5, L5_5, L6_5, L7_5)
              L3_5 = A1_2
              L3_5 = L3_5.addToCollections
              if L3_5 == true then
                L3_5 = A0_2
                L4_5 = L13_4
                L3_5 = L3_5[L4_5]
                L4_5 = L3_5
                L3_5 = L3_5.getContainedCollections
                L3_5 = L3_5(L4_5)
                if L3_5 ~= nil then
                  L4_5 = 1
                  L5_5 = #L3_5
                  L6_5 = 1
                  for L7_5 = L4_5, L5_5, L6_5 do
                    L8_5 = L3_5[L7_5]
                    L9_5 = L8_5
                    L8_5 = L8_5.addPhotos
                    L10_5 = {}
                    L11_5 = L2_5
                    L10_5[1] = L11_5
                    L8_5(L9_5, L10_5)
                  end
                end
              end
            end
            
            L22_4 = {}
            L22_4.timeout = 100
            L18_4(L19_4, L20_4, L21_4, L22_4)
          end
        end
      end
    end
    
    L0_3(L1_3, L2_3)
  end
  
  L2_2(L3_2)
end

L49_1.makePositiveCopy = L50_1
L49_1 = NLPViewController

function L50_1(A0_2, A1_2, A2_2)
  local L3_2, L4_2
  L3_2 = L9_1
  L3_2 = L3_2.startAsyncTask
  
  function L4_2()
    local L0_3, L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3
    L0_3 = A0_2
    L1_3 = _G
    L1_3 = L1_3.activePhotoIndex
    if L0_3 > L1_3 then
      L0_3 = "forward"
      if L0_3 then
        goto lbl_10
      end
    end
    L0_3 = "backward"
    ::lbl_10::
    L1_3 = A0_2
    L2_3 = false
    L3_3 = false
    L4_3 = L19_1
    L4_3 = L4_3.updateActivePhotoMetadata
    L5_3 = A1_2
    L4_3(L5_3)
    while not L2_3 and not L3_3 do
      L4_3 = A0_2
      L5_3 = _G
      L5_3 = L5_3.activePhotos
      L5_3 = #L5_3
      if L4_3 > L5_3 then
        L4_3 = 1
        A0_2 = L4_3
      else
        L4_3 = A0_2
        if L4_3 < 1 then
          L4_3 = _G
          L4_3 = L4_3.activePhotos
          L4_3 = #L4_3
          A0_2 = L4_3
        end
      end
      L4_3 = L19_1
      L4_3 = L4_3.isConverted
      L5_3 = A0_2
      L4_3 = L4_3(L5_3)
      if L4_3 then
        L2_3 = true
      else
        if L0_3 == "forward" then
          L4_3 = A0_2
          L4_3 = L4_3 + 1
          A0_2 = L4_3
        else
          L4_3 = A0_2
          L4_3 = L4_3 - 1
          A0_2 = L4_3
        end
        L4_3 = A0_2
        if L4_3 == L1_3 then
          L4_3 = L24_1
          L5_3 = L4_3
          L4_3 = L4_3.trace
          L6_3 = "No converted photos found in selection"
          L4_3(L5_3, L6_3)
          L3_3 = true
        end
      end
    end
    L4_3 = L12_1
    L5_3 = L4_3
    L4_3 = L4_3.setSelectedPhotos
    L6_3 = _G
    L6_3 = L6_3.activePhotos
    L7_3 = A0_2
    L6_3 = L6_3[L7_3]
    L7_3 = _G
    L7_3 = L7_3.activePhotos
    L4_3(L5_3, L6_3, L7_3)
    L4_3 = A2_2
    L5_3 = A0_2
    L6_3 = " of "
    L7_3 = _G
    L7_3 = L7_3.activePhotos
    L7_3 = #L7_3
    L5_3 = L5_3 .. L6_3 .. L7_3
    L4_3.navigationInfo = L5_3
    L4_3 = L19_1
    L4_3 = L4_3.updateActivePhotoIndex
    L5_3 = A0_2
    L4_3(L5_3)
    L4_3 = createRollDropdown
    L4_3()
    L4_3 = L23_1
    L4_3 = L4_3.handleNavigationChange
    L4_3()
    L4_3 = L9_1
    L4_3 = L4_3.sleep
    L5_3 = 0
    L4_3(L5_3)
    L4_3 = NLPViewController
    L4_3 = L4_3.updateMultipleSettings
    L5_3 = _G
    L5_3 = L5_3.activePhotosMeta
    L6_3 = A0_2
    L5_3 = L5_3[L6_3]
    L5_3 = L5_3.settings
    L6_3 = A1_2
    L4_3(L5_3, L6_3)
  end
  
  L3_2(L4_2)
end

L49_1.navigateToPhoto = L50_1
L49_1 = NLPViewController

function L50_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L1_2 = L22_1
  L2_2 = {}
  L1_2.savedSettings = L2_2
  L1_2 = pairs
  L2_2 = A0_2
  L1_2, L2_2, L3_2 = L1_2(L2_2)
  for L4_2, L5_2 in L1_2, L2_2, L3_2 do
    if L4_2 ~= "source" and L4_2 ~= "colorModel" and L4_2 ~= "saturation" and L4_2 ~= "filmBorder" then
      if L4_2 == "colorPicker" then
        L6_2 = L22_1
        L6_2 = L6_2.savedSettings
        L7_2 = {}
        L9_2 = L5_2
        L8_2 = L5_2.red
        L8_2 = L8_2(L9_2)
        L10_2 = L5_2
        L9_2 = L5_2.green
        L9_2 = L9_2(L10_2)
        L11_2 = L5_2
        L10_2 = L5_2.blue
        L10_2, L11_2 = L10_2(L11_2)
        L7_2[1] = L8_2
        L7_2[2] = L9_2
        L7_2[3] = L10_2
        L7_2[4] = L11_2
        L6_2[L4_2] = L7_2
      else
        L6_2 = L22_1
        L6_2 = L6_2.savedSettings
        L6_2[L4_2] = L5_2
      end
    end
  end
  L1_2 = L22_1
  L2_2 = L22_1
  L2_2 = L2_2.savedSettings
  L1_2.savedSettings = L2_2
  L1_2 = L3_1
  L1_2 = L1_2.messageWithDoNotShow
  L2_2 = {}
  L2_2.message = "Your Settings Saved!"
  L2_2.info = "These settings have been saved as your default. Future conversion will default to this settings. You can also re-load them using the \"load\" command"
  L2_2.actionPrefKey = "OK"
  L1_2(L2_2)
end

L49_1.saveDefault = L50_1
L49_1 = NLPViewController

function L50_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = {}
  L2_2 = "source"
  L3_2 = "previewBuffer"
  L4_2 = "colorModel"
  L5_2 = "filmBorder"
  L6_2 = "saturation"
  L7_2 = "imageType"
  L1_2[1] = L2_2
  L1_2[2] = L3_2
  L1_2[3] = L4_2
  L1_2[4] = L5_2
  L1_2[5] = L6_2
  L1_2[6] = L7_2
  L2_2 = isPositive
  if L2_2 == true then
    L2_2 = L15_1
    L0_2 = L2_2.defaultPositiveSettings
  else
    L2_2 = isMonochrome
    if L2_2 ~= true then
      L2_2 = table
      L2_2 = L2_2.insert
      L3_2 = L1_2
      L4_2 = "rollAnalysis"
      L2_2(L3_2, L4_2)
      L2_2 = L15_1
      L0_2 = L2_2.defaultSettings
    else
      L2_2 = L15_1
      L0_2 = L2_2.defaultMonoSettings
    end
  end
  L2_2 = 1
  L3_2 = #L1_2
  L4_2 = 1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = L1_2[L5_2]
    L7_2 = properties
    L7_2 = L7_2[L6_2]
    L0_2[L6_2] = L7_2
  end
  L2_2 = NLPViewController
  L2_2 = L2_2.updateMultipleSettings
  L3_2 = L0_2
  L4_2 = properties
  L5_2 = true
  L2_2(L3_2, L4_2, L5_2)
end

L49_1.resetSettings = L50_1
L49_1 = NLPViewController

function L50_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = {}
  L2_2 = L22_1
  L2_2 = L2_2.savedSettings
  if L2_2 then
    L2_2 = pairs
    L3_2 = L22_1
    L3_2 = L3_2.savedSettings
    L2_2, L3_2, L4_2 = L2_2(L3_2)
    for L5_2, L6_2 in L2_2, L3_2, L4_2 do
      if L5_2 ~= "source" and L5_2 ~= "colorModel" and L5_2 ~= "saturation" and L5_2 ~= "filmBorder" and L5_2 ~= "colorPicker" then
        if L5_2 == "wbTemp" or L5_2 == "wbTint" then
          L7_2 = L22_1
          L7_2 = L7_2.savedSettings
          L7_2 = L7_2.film
          if L7_2 == "custom" then
            L1_2[L5_2] = L6_2
          end
        else
          L1_2[L5_2] = L6_2
        end
      end
    end
  end
  if L1_2 then
    L2_2 = NLPViewController
    L2_2 = L2_2.updateMultipleSettings
    L3_2 = L1_2
    L4_2 = A0_2
    L2_2(L3_2, L4_2)
  end
end

L49_1.loadDefault = L50_1
L49_1 = NLPViewController

function L50_1()
  local L0_2, L1_2
  L0_2 = L8_1
  L0_2 = L0_2.showClipping
  L0_2()
end

L49_1.showClipping = L50_1
L49_1 = NLPViewController
return L49_1
