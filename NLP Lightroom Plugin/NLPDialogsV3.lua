local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1

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
L10_1 = "LrProgressScope"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrDevelopController"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrHttp"
L11_1 = L11_1(L12_1)
L12_1 = import
L13_1 = "LrUndo"
L12_1 = L12_1(L13_1)
L13_1 = import
L14_1 = "LrPrefs"
L13_1 = L13_1(L14_1)
L13_1 = L13_1.prefsForPlugin
L13_1 = L13_1()
L14_1 = import
L15_1 = "LrColor"
L14_1 = L14_1(L15_1)
L15_1 = require
L16_1 = "UtilV3"
L15_1 = L15_1(L16_1)
L16_1 = require
L17_1 = "NegativePresets"
L16_1 = L16_1(L17_1)
L17_1 = require
L18_1 = "inspect"
L17_1 = L17_1(L18_1)
L18_1 = require
L19_1 = "NLPViewsV3"
L18_1 = L18_1(L19_1)
L19_1 = require
L20_1 = "NLPViewControllerV3"
L19_1 = L19_1(L20_1)
L20_1 = require
L21_1 = "NLPSessionV3"
L20_1 = L20_1(L21_1)
L21_1 = require
L22_1 = "MetadataDefinitionsV3"
L21_1 = L21_1(L22_1)
L22_1 = L21_1.metadataFieldsForPhotos
L23_1 = L8_1
L24_1 = "NLPDialogs"
L23_1 = L23_1(L24_1)
L24_1 = _G
L24_1 = L24_1.dev
if L24_1 == true then
  L25_1 = L23_1
  L24_1 = L23_1.enable
  L26_1 = "logfile"
  L24_1(L25_1, L26_1)
else
  L25_1 = L23_1
  L24_1 = L23_1.disable
  L24_1(L25_1)
end
L24_1 = 1
undoCount = L24_1
L24_1 = {}
propertiesObserverTable = L24_1
L24_1 = false
L25_1 = L10_1.setMultipleAdjustmentThreshold
L26_1 = 600
L25_1(L26_1)
L25_1 = L10_1.setTrackingDelay
L26_1 = 600
L25_1(L26_1)
L25_1 = L1_1.activeCatalog
L25_1 = L25_1()
L26_1 = {}
NLPDialogs = L26_1

function L26_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L1_2 = L3_1
  L1_2 = L1_2.makePropertyTable
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  presetTable = L1_2
  L1_2 = presetTable
  L1_2.selectedPreset = "Custom"
  L1_2 = presetTable
  L2_2 = {}
  L3_2 = {}
  L3_2.title = "Custom"
  L3_2.value = "Custom"
  L4_2 = {}
  L4_2.title = "C-Print Portrait Tones"
  L4_2.value = "C-Print Portrait Tones"
  L5_2 = {}
  L5_2.title = "Green-shadow Noritsu"
  L5_2.value = "Green-tint Noritsu"
  L6_2 = {}
  L6_2.title = "Moody Cinestill 800"
  L6_2.value = "Cinematic Soft"
  L7_2 = {}
  L7_2.title = "Warm Soft Portra"
  L7_2.value = "Warm Portra"
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  L2_2[4] = L6_2
  L2_2[5] = L7_2
  L1_2.presets = L2_2
  L1_2 = presetTable
  L2_2 = {}
  L3_2 = {}
  L3_2.separator = true
  L4_2 = {}
  L4_2.title = "Save as new preset..."
  L4_2.value = "save"
  L5_2 = {}
  L5_2.title = "Delete preset..."
  L5_2.value = "delete"
  L6_2 = {}
  L6_2.title = "Update preset..."
  L6_2.value = "update"
  L7_2 = {}
  L7_2.title = "Set as default preset..."
  L7_2.value = "default"
  L2_2[1] = L3_2
  L2_2[2] = L4_2
  L2_2[3] = L5_2
  L2_2[4] = L6_2
  L2_2[5] = L7_2
  L1_2.actions = L2_2
end

makePresetObservableTable = L26_1

function L26_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2
  L1_2 = {}
  L2_2 = {}
  L3_2 = {}
  L4_2 = _G
  L5_2 = {}
  L4_2.orderedPresetArray = L5_2
  L4_2 = _G
  L4_2 = L4_2.nlpPresets
  if L4_2 then
    L4_2 = pairs
    L5_2 = _G
    L5_2 = L5_2.nlpPresets
    L4_2, L5_2, L6_2 = L4_2(L5_2)
    for L7_2, L8_2 in L4_2, L5_2, L6_2 do
      L9_2 = L8_2.presetName
      L10_2 = L8_2.id
      L11_2 = L8_2.group
      L12_2 = L8_2.isMonochrome
      L13_2 = L3_2[L11_2]
      if not L13_2 then
        L13_2 = {}
        L3_2[L11_2] = L13_2
      end
      L13_2 = table
      L13_2 = L13_2.insert
      L14_2 = L3_2[L11_2]
      L15_2 = {}
      L15_2.title = L9_2
      L15_2.value = L10_2
      L13_2(L14_2, L15_2)
    end
    L4_2 = {}
    L5_2 = pairs
    L6_2 = L3_2
    L5_2, L6_2, L7_2 = L5_2(L6_2)
    for L8_2 in L5_2, L6_2, L7_2 do
      L9_2 = table
      L9_2 = L9_2.insert
      L10_2 = L4_2
      L11_2 = L8_2
      L9_2(L10_2, L11_2)
    end
    L5_2 = table
    L5_2 = L5_2.sort
    L6_2 = L4_2
    L5_2(L6_2)
    L5_2 = 1
    L6_2 = #L4_2
    L7_2 = 1
    for L8_2 = L5_2, L6_2, L7_2 do
      L9_2 = L4_2[L8_2]
      L10_2 = L15_1
      L10_2 = L10_2.sortByKey
      L11_2 = L3_2[L9_2]
      L12_2 = "title"
      L10_2 = L10_2(L11_2, L12_2)
      if A0_2 ~= nil then
        L11_2 = 1
        L12_2 = #L10_2
        L13_2 = 1
        for L14_2 = L11_2, L12_2, L13_2 do
          L15_2 = L10_2[L14_2]
          L15_2 = L15_2.value
          if L15_2 == A0_2 then
            L15_2 = L10_2[L14_2]
            L16_2 = "*"
            L17_2 = L10_2[L14_2]
            L17_2 = L17_2.title
            L16_2 = L16_2 .. L17_2
            L15_2.title = L16_2
          end
        end
      end
      L11_2 = {}
      L11_2.title = L9_2
      L12_2 = 1
      L13_2 = #L10_2
      L14_2 = 1
      for L15_2 = L12_2, L13_2, L14_2 do
        L16_2 = table
        L16_2 = L16_2.insert
        L17_2 = L11_2
        L18_2 = {}
        L19_2 = L10_2[L15_2]
        L19_2 = L19_2.title
        L18_2.title = L19_2
        L19_2 = L10_2[L15_2]
        L19_2 = L19_2.value
        L18_2.value = L19_2
        L16_2(L17_2, L18_2)
        L16_2 = table
        L16_2 = L16_2.insert
        L17_2 = orderedPresetArray
        L18_2 = L10_2[L15_2]
        L18_2 = L18_2.value
        L16_2(L17_2, L18_2)
      end
      L12_2 = table
      L12_2 = L12_2.insert
      L13_2 = L1_2
      L14_2 = L11_2
      L12_2(L13_2, L14_2)
    end
    L5_2 = uiSettings
    L5_2.presetDropdown = L1_2
  end
end

createPresetTable = L26_1

function L26_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L2_2 = uiSettings
  L3_2 = {}
  L2_2.presetActions = L3_2
  L2_2 = _G
  L2_2 = L2_2.nlpPresets
  L3_2 = A1_2.selectedPreset
  L2_2 = L2_2[L3_2]
  if L2_2 ~= nil then
    L2_2 = _G
    L2_2 = L2_2.nlpPresets
    L3_2 = A1_2.selectedPreset
    L2_2 = L2_2[L3_2]
    L2_2 = L2_2.presetName
    if A0_2 == "update" then
      L3_2 = uiSettings
      L3_2.presetActions = nil
      L3_2 = uiSettings
      L4_2 = {}
      L5_2 = {}
      L5_2.separator = true
      L6_2 = {}
      L7_2 = "Update \""
      L8_2 = L2_2
      L9_2 = "\"..."
      L7_2 = L7_2 .. L8_2 .. L9_2
      L6_2.title = L7_2
      L6_2.value = "update"
      L7_2 = {}
      L7_2.title = "Make Default..."
      L7_2.value = "default"
      L8_2 = {}
      L8_2.title = "Reload..."
      L8_2.value = "reload"
      L9_2 = {}
      L9_2.title = "Delete..."
      L9_2.value = "delete"
      L10_2 = {}
      L10_2.separator = true
      L11_2 = {}
      L11_2.title = "Create New Preset..."
      L11_2.value = "save"
      L4_2[1] = L5_2
      L4_2[2] = L6_2
      L4_2[3] = L7_2
      L4_2[4] = L8_2
      L4_2[5] = L9_2
      L4_2[6] = L10_2
      L4_2[7] = L11_2
      L3_2.presetActions = L4_2
    elseif A0_2 == "new" then
      L3_2 = uiSettings
      L4_2 = {}
      L5_2 = {}
      L5_2.separator = true
      L6_2 = {}
      L6_2.title = "Make Default..."
      L6_2.value = "default"
      L7_2 = {}
      L7_2.title = "Delete..."
      L7_2.value = "delete"
      L8_2 = {}
      L8_2.separator = true
      L9_2 = {}
      L9_2.title = "Create New Preset..."
      L9_2.value = "save"
      L4_2[1] = L5_2
      L4_2[2] = L6_2
      L4_2[3] = L7_2
      L4_2[4] = L8_2
      L4_2[5] = L9_2
      L3_2.presetActions = L4_2
    elseif A0_2 == "empty" then
      L3_2 = uiSettings
      L4_2 = {}
      L5_2 = {}
      L5_2.separator = true
      L6_2 = {}
      L6_2.title = "Create New Preset..."
      L6_2.value = "save"
      L4_2[1] = L5_2
      L4_2[2] = L6_2
      L3_2.presetActions = L4_2
    end
  else
    L2_2 = uiSettings
    L3_2 = {}
    L4_2 = {}
    L4_2.title = "Custom"
    L5_2 = {}
    L5_2.title = "Custom"
    L5_2.value = "custom"
    L4_2[1] = L5_2
    L5_2 = {}
    L5_2.separator = true
    L6_2 = {}
    L6_2.title = "Create New Preset..."
    L6_2.value = "save"
    L3_2[1] = L4_2
    L3_2[2] = L5_2
    L3_2[3] = L6_2
    L2_2.presetActions = L3_2
  end
end

updatePresetActions = L26_1

function L26_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2
  L0_2 = {}
  L1_2 = {}
  L2_2 = {}
  L3_2 = {}
  L3_2.title = "This image only"
  L3_2.value = "individual"
  L4_2 = {}
  L4_2.separator = true
  L5_2 = table
  L5_2 = L5_2.insert
  L6_2 = L0_2
  L7_2 = L3_2
  L5_2(L6_2, L7_2)
  L5_2 = table
  L5_2 = L5_2.insert
  L6_2 = L0_2
  L7_2 = L4_2
  L5_2(L6_2, L7_2)
  L5_2 = _G
  L5_2 = L5_2.activePhotosMeta
  L6_2 = activePhotoIndex
  L5_2 = L5_2[L6_2]
  L5_2 = L5_2.roll
  if L5_2 ~= nil then
    L6_2 = L5_2.rollName
    L7_2 = L5_2.ID
    L8_2 = table
    L8_2 = L8_2.insert
    L9_2 = L0_2
    L10_2 = {}
    L10_2.title = L6_2
    L10_2.value = L7_2
    L8_2(L9_2, L10_2)
    L8_2 = table
    L8_2 = L8_2.insert
    L9_2 = L0_2
    L10_2 = L4_2
    L8_2(L9_2, L10_2)
  end
  L6_2 = _G
  L6_2 = L6_2.rolls
  if L6_2 then
  end
  L6_2 = _G
  L6_2 = L6_2.rolls
  if L6_2 then
    L6_2 = pairs
    L7_2 = _G
    L7_2 = L7_2.rolls
    L6_2, L7_2, L8_2 = L6_2(L7_2)
    for L9_2, L10_2 in L6_2, L7_2, L8_2 do
      if L9_2 ~= "groups" then
        L11_2 = L10_2.rollName
        L12_2 = L10_2.ID
        L13_2 = L10_2.groupName
        if not L13_2 then
          L13_2 = "User"
        end
        L14_2 = L2_2[L13_2]
        if not L14_2 then
          L14_2 = {}
          L2_2[L13_2] = L14_2
        end
        L14_2 = table
        L14_2 = L14_2.insert
        L15_2 = L2_2[L13_2]
        L16_2 = {}
        L16_2.title = L11_2
        L16_2.value = L12_2
        L14_2(L15_2, L16_2)
      end
    end
    L6_2 = {}
    L7_2 = pairs
    L8_2 = L2_2
    L7_2, L8_2, L9_2 = L7_2(L8_2)
    for L10_2 in L7_2, L8_2, L9_2 do
      L11_2 = table
      L11_2 = L11_2.insert
      L12_2 = L6_2
      L13_2 = L10_2
      L11_2(L12_2, L13_2)
    end
    L7_2 = table
    L7_2 = L7_2.sort
    L8_2 = L6_2
    L7_2(L8_2)
    L7_2 = 1
    L8_2 = #L6_2
    L9_2 = 1
    for L10_2 = L7_2, L8_2, L9_2 do
      L11_2 = L6_2[L10_2]
      L12_2 = _G
      L12_2 = L12_2.rolls
      L12_2 = L12_2.groups
      L12_2 = L12_2[L11_2]
      if L12_2 == true then
        L12_2 = L15_1
        L12_2 = L12_2.sortByKey
        L13_2 = L2_2[L11_2]
        L14_2 = "title"
        L12_2 = L12_2(L13_2, L14_2)
        L13_2 = {}
        L13_2.title = L11_2
        L14_2 = 1
        L15_2 = #L12_2
        L16_2 = 1
        for L17_2 = L14_2, L15_2, L16_2 do
          L18_2 = table
          L18_2 = L18_2.insert
          L19_2 = L13_2
          L20_2 = {}
          L21_2 = L12_2[L17_2]
          L21_2 = L21_2.title
          L20_2.title = L21_2
          L21_2 = L12_2[L17_2]
          L21_2 = L21_2.value
          L20_2.value = L21_2
          L18_2(L19_2, L20_2)
        end
        L14_2 = table
        L14_2 = L14_2.insert
        L15_2 = L0_2
        L16_2 = L13_2
        L14_2(L15_2, L16_2)
      end
    end
  end
  L6_2 = properties
  L6_2 = L6_2.rollAnalysis
  if L6_2 ~= "individual" then
    L6_2 = properties
    L6_2 = L6_2.rollAnalysis
    if L6_2 ~= "add" then
      L6_2 = properties
      L6_2 = L6_2.rollAnalysis
      if L6_2 ~= "manage" then
        L6_2 = properties
        L6_2 = L6_2.rollAnalysis
        if L6_2 ~= nil then
          L6_2 = {}
          L6_2.title = "Rename Roll..."
          L6_2.value = "edit"
          L7_2 = {}
          L7_2.title = "Delete Roll..."
          L7_2.value = "delete"
          L8_2 = table
          L8_2 = L8_2.insert
          L9_2 = L0_2
          L10_2 = L4_2
          L8_2(L9_2, L10_2)
          L8_2 = table
          L8_2 = L8_2.insert
          L9_2 = L0_2
          L10_2 = L6_2
          L8_2(L9_2, L10_2)
          L8_2 = table
          L8_2 = L8_2.insert
          L9_2 = L0_2
          L10_2 = L7_2
          L8_2(L9_2, L10_2)
        end
      end
    end
  end
  L6_2 = {}
  L6_2.title = "New Roll..."
  L6_2.value = "add"
  L7_2 = {}
  L7_2.title = "Manage Groups..."
  L7_2.value = "manage"
  L8_2 = table
  L8_2 = L8_2.insert
  L9_2 = L0_2
  L10_2 = L4_2
  L8_2(L9_2, L10_2)
  L8_2 = table
  L8_2 = L8_2.insert
  L9_2 = L0_2
  L10_2 = L6_2
  L8_2(L9_2, L10_2)
  L8_2 = table
  L8_2 = L8_2.insert
  L9_2 = L0_2
  L10_2 = L7_2
  L8_2(L9_2, L10_2)
  L8_2 = uiSettings
  L8_2.rollDropdown = L0_2
end

createRollDropdown = L26_1
L26_1 = NLPDialogs

function L27_1()
  local L0_2, L1_2
  L0_2 = L7_1
  L0_2 = L0_2.startAsyncTask
  
  function L1_2()
    local L0_3, L1_3, L2_3, L3_3
    L1_3 = L4_1
    L1_3 = L1_3.postAsyncTaskWithContext
    L2_3 = "negativeDialog"
    
    function L3_3(A0_4)
      local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4, L11_4
      L1_4 = L4_1
      L1_4 = L1_4.callWithContext
      L2_4 = "loadingDialog"
      
      function L3_4(A0_5)
        local L1_5, L2_5, L3_5, L4_5, L5_5, L6_5, L7_5, L8_5, L9_5, L10_5, L11_5, L12_5, L13_5, L14_5, L15_5, L16_5, L17_5, L18_5, L19_5, L20_5, L21_5, L22_5, L23_5, L24_5, L25_5, L26_5, L27_5, L28_5, L29_5, L30_5, L31_5, L32_5, L33_5, L34_5, L35_5, L36_5, L37_5, L38_5, L39_5, L40_5, L41_5, L42_5, L43_5, L44_5, L45_5, L46_5, L47_5, L48_5, L49_5, L50_5, L51_5, L52_5
        L1_5 = {}
        L1_5.title = "Loading Negative Lab Pro..."
        L1_5.caption = "Just a minute..."
        L1_5.functionContext = A0_5
        L2_5 = L5_1
        L2_5 = L2_5.showModalProgressDialog
        L3_5 = L1_5
        L2_5 = L2_5(L3_5)
        progressScopeLoadingSession = L2_5
        L2_5 = progressScopeLoadingSession
        L3_5 = L2_5
        L2_5 = L2_5.setIndeterminate
        L2_5(L3_5)
        L2_5 = progressScopeLoadingSession
        L3_5 = L2_5
        L2_5 = L2_5.setCaption
        L4_5 = "Loading settings and metadata. This could take a minute..."
        L2_5(L3_5, L4_5)
        L2_5 = L20_1
        L2_5 = L2_5.startSession
        L2_5()
        L2_5 = L6_1
        L2_5 = L2_5.osFactory
        L2_5 = L2_5()
        L0_3 = L2_5
        L2_5 = L3_1
        L2_5 = L2_5.makePropertyTable
        L3_5 = A0_4
        L2_5 = L2_5(L3_5)
        properties = L2_5
        L2_5 = {}
        L3_5 = "exposure"
        L4_5 = "brightness"
        L5_5 = "highlights"
        L6_5 = "shadows"
        L7_5 = "whites"
        L8_5 = "blacks"
        L9_5 = "glow"
        L10_5 = "fade"
        L11_5 = "autoColorIsEnabled"
        L12_5 = "autoColorLevel"
        L13_5 = "autoToneIsEnabled"
        L14_5 = "autoToneLevel"
        L15_5 = "toneProfile"
        L16_5 = "temp"
        L17_5 = "tint"
        L18_5 = "cyan"
        L19_5 = "highlightTemp"
        L20_5 = "highlightTint"
        L21_5 = "highlightCyan"
        L22_5 = "shadowTemp"
        L23_5 = "shadowTint"
        L24_5 = "shadowCyan"
        L25_5 = "colorModel"
        L26_5 = "saturation"
        L27_5 = "sharpening"
        L28_5 = "filmBorder"
        L29_5 = "previewBuffer"
        L30_5 = "source"
        L31_5 = "filmHue"
        L32_5 = "hueStrength"
        L33_5 = "film"
        L34_5 = "colorPicker"
        L35_5 = "wb"
        L36_5 = "wbTemp"
        L37_5 = "wbTint"
        L38_5 = "shadowRange"
        L39_5 = "highlightRange"
        L40_5 = "enhancedProfile"
        L41_5 = "enhancedProfileStrength"
        L42_5 = "postSaturation"
        L43_5 = "curveResolution"
        L44_5 = "curveResolutionType"
        L45_5 = "layerOrder"
        L46_5 = "wbTonality"
        L47_5 = "wbMethod"
        L48_5 = "enginePreset"
        L49_5 = "rollAnalysis"
        L50_5 = "rollMode"
        L51_5 = "convertAsRoll"
        L52_5 = "clipMethod"
        L2_5[1] = L3_5
        L2_5[2] = L4_5
        L2_5[3] = L5_5
        L2_5[4] = L6_5
        L2_5[5] = L7_5
        L2_5[6] = L8_5
        L2_5[7] = L9_5
        L2_5[8] = L10_5
        L2_5[9] = L11_5
        L2_5[10] = L12_5
        L2_5[11] = L13_5
        L2_5[12] = L14_5
        L2_5[13] = L15_5
        L2_5[14] = L16_5
        L2_5[15] = L17_5
        L2_5[16] = L18_5
        L2_5[17] = L19_5
        L2_5[18] = L20_5
        L2_5[19] = L21_5
        L2_5[20] = L22_5
        L2_5[21] = L23_5
        L2_5[22] = L24_5
        L2_5[23] = L25_5
        L2_5[24] = L26_5
        L2_5[25] = L27_5
        L2_5[26] = L28_5
        L2_5[27] = L29_5
        L2_5[28] = L30_5
        L2_5[29] = L31_5
        L2_5[30] = L32_5
        L2_5[31] = L33_5
        L2_5[32] = L34_5
        L2_5[33] = L35_5
        L2_5[34] = L36_5
        L2_5[35] = L37_5
        L2_5[36] = L38_5
        L2_5[37] = L39_5
        L2_5[38] = L40_5
        L2_5[39] = L41_5
        L2_5[40] = L42_5
        L2_5[41] = L43_5
        L2_5[42] = L44_5
        L2_5[43] = L45_5
        L2_5[44] = L46_5
        L2_5[45] = L47_5
        L2_5[46] = L48_5
        L2_5[47] = L49_5
        L2_5[48] = L50_5
        L2_5[49] = L51_5
        L2_5[50] = L52_5
        L3_5 = "imageType"
        L2_5[51] = L3_5
        L3_5 = L3_1
        L3_5 = L3_5.makePropertyTable
        L4_5 = A0_4
        L3_5 = L3_5(L4_5)
        uiSettings = L3_5
        L3_5 = createRollDropdown
        L3_5()
        L3_5 = uiSettings
        L4_5 = _G
        L4_5 = L4_5.isConverted
        L3_5.isConverted = L4_5
        L3_5 = uiSettings
        L3_5.colorModelIsEnabled = true
        L3_5 = L3_1
        L3_5 = L3_5.makePropertyTable
        L4_5 = A0_4
        L3_5 = L3_5(L4_5)
        viewSettings = L3_5
        L3_5 = viewSettings
        L3_5.showingBefore = false
        L3_5 = L13_1
        L3_5 = L3_5.ui
        if L3_5 == nil then
          L3_5 = L13_1
          L4_5 = {}
          L3_5.ui = L4_5
        end
        L3_5 = uiSettings
        L3_5.convertButtonIsEnabled = true
        L3_5 = activePhotos
        L3_5 = #L3_5
        if 1 < L3_5 then
          L3_5 = uiSettings
          L4_5 = "CONVERT "
          L5_5 = activePhotos
          L5_5 = #L5_5
          L6_5 = " NEGATIVES"
          L4_5 = L4_5 .. L5_5 .. L6_5
          L3_5.convertButtonTitle = L4_5
          L3_5 = uiSettings
          L4_5 = "UN-CONVERT "
          L5_5 = activePhotos
          L5_5 = #L5_5
          L6_5 = " NEGATIVES"
          L4_5 = L4_5 .. L5_5 .. L6_5
          L3_5.unconvertButtonTitle = L4_5
        else
          L3_5 = uiSettings
          L3_5.convertButtonTitle = "CONVERT 1 NEGATIVE"
          L3_5 = uiSettings
          L3_5.unconvertButtonTitle = "UN-CONVERT 1 NEGATIVE"
        end
        L3_5 = uiSettings
        L3_5.unconvertIsVisible = false
        L3_5 = activePhotos
        L3_5 = #L3_5
        if 1 < L3_5 then
          L3_5 = uiSettings
          L3_5.syncIsVisible = true
          L3_5 = uiSettings
          L4_5 = activePhotoIndex
          L5_5 = " of "
          L6_5 = activePhotos
          L6_5 = #L6_5
          L4_5 = L4_5 .. L5_5 .. L6_5
          L3_5.navigationInfo = L4_5
        else
          L3_5 = uiSettings
          L3_5.syncIsVisible = false
          L3_5 = uiSettings
          L3_5.navigationInfo = "-"
        end
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.alwaysEnableCopy
        if not L4_5 then
          L4_5 = false
        end
        L3_5.tiffCopy = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.fileType
        if not L4_5 then
          L4_5 = "tif"
        end
        L3_5.fileType = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.outputType
        if not L4_5 then
          L4_5 = "stack"
        end
        L3_5.outputType = L4_5
        L3_5 = uiSettings
        L3_5.masterTabView = "convert"
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.jpgQuality
        if not L4_5 then
          L4_5 = 100
        end
        L3_5.jpgQuality = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.tifBits
        if not L4_5 then
          L4_5 = "16"
        end
        L3_5.tifBits = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.tifCompress
        if not L4_5 then
          L4_5 = "compressionMethod_ZIP"
        end
        L3_5.tifCompress = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.exportColorspace
        if not L4_5 then
          L4_5 = "sRGB"
        end
        L3_5.exportColorspace = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.exportSubfolderPath
        if not L4_5 then
          L4_5 = "positives"
        end
        L3_5.exportSubfolderPath = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.addToCollections
        if not L4_5 then
          L4_5 = true
        end
        L3_5.addToCollections = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.alwaysEnableCopy
        if not L4_5 then
          L4_5 = false
        end
        L3_5.alwaysEnableCopy = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.overwriteOldCopies
        if not L4_5 then
          L4_5 = false
        end
        L3_5.overwriteOldCopies = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.excludeMetadata
        if not L4_5 then
          L4_5 = false
        end
        L3_5.excludeMetadata = L4_5
        L3_5 = uiSettings
        L3_5.undoIsVisible = false
        L3_5 = uiSettings
        L3_5.redoIsVisible = false
        L3_5 = L13_1
        L3_5 = L3_5.ui
        L3_5 = L3_5.colorTabView
        if L3_5 == nil then
          L3_5 = L13_1
          L3_5 = L3_5.ui
          L3_5.colorTabView = "film"
        end
        L3_5 = L13_1
        L3_5 = L3_5.ui
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.colorTabView
        L3_5.colorTabView = L4_5
        L3_5 = uiSettings
        L4_5 = L13_1
        L4_5 = L4_5.ui
        L4_5 = L4_5.colorTabView
        L3_5.colorTabView = L4_5
        L3_5 = {}
        L4_5 = "colorTabView"
        L5_5 = "fileType"
        L6_5 = "outputType"
        L7_5 = "jpgQuality"
        L8_5 = "tifBits"
        L9_5 = "tifCompress"
        L10_5 = "exportColorspace"
        L11_5 = "exportSubfolderPath"
        L12_5 = "addToCollections"
        L13_5 = "alwaysEnableCopy"
        L14_5 = "overwriteOldCopies"
        L15_5 = "excludeMetadata"
        L3_5[1] = L4_5
        L3_5[2] = L5_5
        L3_5[3] = L6_5
        L3_5[4] = L7_5
        L3_5[5] = L8_5
        L3_5[6] = L9_5
        L3_5[7] = L10_5
        L3_5[8] = L11_5
        L3_5[9] = L12_5
        L3_5[10] = L13_5
        L3_5[11] = L14_5
        L3_5[12] = L15_5
        L4_5 = 1
        L5_5 = #L3_5
        L6_5 = 1
        for L7_5 = L4_5, L5_5, L6_5 do
          L8_5 = uiSettings
          L9_5 = L8_5
          L8_5 = L8_5.addObserver
          L10_5 = L3_5[L7_5]
          
          function L11_5(A0_6)
            local L1_6, L2_6, L3_6, L4_6, L5_6
            L1_6 = L13_1
            L1_6 = L1_6.ui
            L2_6 = L3_5
            L3_6 = L7_5
            L2_6 = L2_6[L3_6]
            L3_6 = L3_5
            L4_6 = L7_5
            L3_6 = L3_6[L4_6]
            L3_6 = A0_6[L3_6]
            L1_6[L2_6] = L3_6
            L1_6 = L13_1
            L1_6 = L1_6.ui
            L2_6 = L3_5
            L3_6 = L7_5
            L2_6 = L2_6[L3_6]
            L3_6 = L13_1
            L3_6 = L3_6.ui
            L4_6 = L3_5
            L5_6 = L7_5
            L4_6 = L4_6[L5_6]
            L3_6 = L3_6[L4_6]
            L1_6[L2_6] = L3_6
          end
          
          L8_5(L9_5, L10_5, L11_5)
        end
        L4_5 = viewSettings
        L5_5 = L4_5
        L4_5 = L4_5.addObserver
        L6_5 = showingBefore
        
        function L7_5(A0_6, A1_6, A2_6)
          local L3_6, L4_6, L5_6, L6_6, L7_6
          L3_6 = L19_1
          L3_6 = L3_6.toggleBefore
          L4_6 = uiSettings
          L5_6 = A1_6
          L6_6 = A2_6
          L7_6 = properties
          L3_6(L4_6, L5_6, L6_6, L7_6)
        end
        
        L4_5(L5_5, L6_5, L7_5)
        L4_5 = activePhotosMeta
        L5_5 = activePhotoIndex
        L4_5 = L4_5[L5_5]
        L4_5 = L4_5.redChannel
        L4_5 = L4_5.whitePointOrigin
        if L4_5 ~= nil then
          L4_5 = activePhotosMeta
          L5_5 = activePhotoIndex
          L4_5 = L4_5[L5_5]
          L4_5 = L4_5.settings
          L4_5 = L4_5.film
          if L4_5 ~= "custom" then
            L4_5 = activePhotosMeta
            L5_5 = activePhotoIndex
            L4_5 = L4_5[L5_5]
            L4_5 = L4_5.settings
            L4_5 = L4_5.film
            if L4_5 ~= "none" then
              L4_5 = L19_1
              L4_5 = L4_5.updateFilm
              L5_5 = activePhotosMeta
              L6_5 = activePhotoIndex
              L5_5 = L5_5[L6_5]
              L5_5 = L5_5.settings
              L6_5 = activePhotoIndex
              L4_5 = L4_5(L5_5, L6_5)
              L5_5 = activePhotosMeta
              L6_5 = activePhotoIndex
              L5_5 = L5_5[L6_5]
              L5_5 = L5_5.settings
              L6_5 = L4_5[3]
              L6_5 = -L6_5
              L7_5 = _G
              L7_5 = L7_5.tempMultiplier
              L6_5 = L6_5 / L7_5
              L5_5.wbTemp = L6_5
              L5_5 = activePhotosMeta
              L6_5 = activePhotoIndex
              L5_5 = L5_5[L6_5]
              L5_5 = L5_5.settings
              L6_5 = L4_5[2]
              L6_5 = -L6_5
              L7_5 = _G
              L7_5 = L7_5.tempMultiplier
              L6_5 = L6_5 / L7_5
              L5_5.wbTint = L6_5
            end
          end
        end
        L4_5 = pairs
        L5_5 = activePhotosMeta
        L6_5 = activePhotoIndex
        L5_5 = L5_5[L6_5]
        L5_5 = L5_5.settings
        L4_5, L5_5, L6_5 = L4_5(L5_5)
        for L7_5, L8_5 in L4_5, L5_5, L6_5 do
          L9_5 = properties
          L9_5[L7_5] = L8_5
          L9_5 = L23_1
          L10_5 = L9_5
          L9_5 = L9_5.trace
          L11_5 = "Loading property: "
          L12_5 = L7_5
          L11_5 = L11_5 .. L12_5
          L9_5(L10_5, L11_5)
        end
        L4_5 = properties
        L4_5 = L4_5.colorPicker
        if L4_5 ~= nil then
          L4_5 = properties
          L5_5 = L15_1
          L5_5 = L5_5.arrayToColor
          L6_5 = properties
          L6_5 = L6_5.colorPicker
          L5_5 = L5_5(L6_5)
          L4_5.colorPicker = L5_5
        end
        L4_5 = activePhotos
        L4_5 = #L4_5
        if 1 < L4_5 then
          L4_5 = L19_1
          L4_5 = L4_5.updateAllUnconverted
          L4_5()
          L4_5 = activePhotosMeta
          L5_5 = _G
          L5_5 = L5_5.activePhotoIndex
          L4_5 = L4_5[L5_5]
          L4_5 = L4_5.redChannel
          L4_5 = L4_5.whitePointOrigin
          if L4_5 ~= nil then
            L4_5 = {}
            L5_5 = 1
            L6_5 = activePhotos
            L6_5 = #L6_5
            L7_5 = 1
            for L8_5 = L5_5, L6_5, L7_5 do
              L9_5 = {}
              L10_5 = activePhotos
              L10_5 = L10_5[L8_5]
              L9_5.photo = L10_5
              L10_5 = {}
              L11_5 = activePhotosMeta
              L11_5 = L11_5[L8_5]
              L11_5 = L11_5.redChannel
              L12_5 = activePhotosMeta
              L12_5 = L12_5[L8_5]
              L12_5 = L12_5.greenChannel
              L13_5 = activePhotosMeta
              L13_5 = L13_5[L8_5]
              L13_5 = L13_5.blueChannel
              L10_5[1] = L11_5
              L10_5[2] = L12_5
              L10_5[3] = L13_5
              L9_5.rgbInput = L10_5
              L10_5 = activePhotosMeta
              L10_5 = L10_5[L8_5]
              L10_5 = L10_5.settings
              L9_5.properties = L10_5
              L10_5 = table
              L10_5 = L10_5.insert
              L11_5 = L4_5
              L12_5 = L9_5
              L10_5(L11_5, L12_5)
            end
            L5_5 = #L4_5
            if 1 <= L5_5 then
              L5_5 = L19_1
              L5_5 = L5_5.updateAllAdjustments
              L6_5 = L4_5
              L7_5 = uiSettings
              L5_5(L6_5, L7_5)
            end
          end
        else
          L4_5 = activePhotosMeta
          L5_5 = activePhotoIndex
          L4_5 = L4_5[L5_5]
          L4_5 = L4_5.redChannel
          L4_5 = L4_5.whitePointOrigin
          if L4_5 ~= nil then
            L4_5 = L15_1
            L4_5 = L4_5.copyObservableTable
            L5_5 = properties
            L4_5 = L4_5(L5_5)
            L5_5 = L19_1
            L5_5 = L5_5.updateAllAdjustments
            L6_5 = {}
            L7_5 = {}
            L8_5 = activePhoto
            L7_5.photo = L8_5
            L8_5 = {}
            L9_5 = activePhotosMeta
            L10_5 = activePhotoIndex
            L9_5 = L9_5[L10_5]
            L9_5 = L9_5.redChannel
            L10_5 = activePhotosMeta
            L11_5 = activePhotoIndex
            L10_5 = L10_5[L11_5]
            L10_5 = L10_5.greenChannel
            L11_5 = activePhotosMeta
            L12_5 = activePhotoIndex
            L11_5 = L11_5[L12_5]
            L11_5 = L11_5.blueChannel
            L8_5[1] = L9_5
            L8_5[2] = L10_5
            L8_5[3] = L11_5
            L7_5.rgbInput = L8_5
            L8_5 = activePhotosMeta
            L9_5 = activePhotoIndex
            L8_5 = L8_5[L9_5]
            L8_5 = L8_5.settings
            L7_5.properties = L8_5
            L6_5[1] = L7_5
            L7_5 = uiSettings
            L5_5(L6_5, L7_5)
          else
            L4_5 = L19_1
            L4_5 = L4_5.updateAllUnconverted
            L4_5()
          end
        end
        L4_5 = nil
        L5_5 = activePhotosMeta
        L6_5 = activePhotoIndex
        L5_5 = L5_5[L6_5]
        L5_5 = L5_5.redChannel
        L5_5 = L5_5.whitePointOrigin
        if L5_5 ~= nil then
          L4_5 = true
          L5_5 = uiSettings
          L5_5.convertButtonIsEnabled = false
          L5_5 = uiSettings
          L5_5.colorModelIsEnabled = false
          L5_5 = uiSettings
          L5_5.convertButtonTitle = "Converted"
          L5_5 = uiSettings
          L5_5.unconvertIsVisible = true
          L5_5 = uiSettings
          L5_5.masterTabView = "edit"
        else
          L4_5 = false
        end
        L5_5 = L20_1
        L5_5 = L5_5.useEnhancedProfiles
        L6_5 = properties
        L5_5, L6_5, L7_5 = L5_5(L6_5)
        L8_5 = uiSettings
        L8_5.useEnhancedProfiles = L5_5
        L8_5 = uiSettings
        L8_5.enhancedProfileError = L7_5
        L8_5 = pairs
        L9_5 = L16_1
        L9_5 = L9_5.defaultSettings
        L8_5, L9_5, L10_5 = L8_5(L9_5)
        for L11_5, L12_5 in L8_5, L9_5, L10_5 do
          L13_5 = properties
          L14_5 = L13_5
          L13_5 = L13_5.addObserver
          L15_5 = L11_5
          L16_5 = propertiesObserverTable
          
          function L17_5(A0_6, A1_6, A2_6, A3_6)
            local L4_6, L5_6, L6_6, L7_6, L8_6, L9_6, L10_6
            L4_6 = L19_1
            L4_6 = L4_6.propertiesChanged
            L5_6 = A0_6
            L6_6 = A1_6
            L7_6 = A2_6
            L8_6 = A3_6
            L9_6 = uiSettings
            L10_6 = viewSettings
            L4_6(L5_6, L6_6, L7_6, L8_6, L9_6, L10_6)
          end
          
          L13_5(L14_5, L15_5, L16_5, L17_5)
        end
        L8_5 = createPresetTable
        L8_5()
        L8_5 = updatePresetActions
        L9_5 = "new"
        L10_5 = properties
        L8_5(L9_5, L10_5)
      end
      
      L1_4(L2_4, L3_4)
      L1_4 = L7_1
      L1_4 = L1_4.sleep
      L2_4 = 0
      L1_4(L2_4)
      L1_4 = L18_1
      L1_4 = L1_4.masterView
      L2_4 = L0_3
      L3_4 = properties
      L4_4 = uiSettings
      L5_4 = viewSettings
      L1_4 = L1_4(L2_4, L3_4, L4_4, L5_4)
      L2_4 = L18_1
      L2_4 = L2_4.accessoryView
      L3_4 = L0_3
      L4_4 = properties
      L5_4 = uiSettings
      L6_4 = viewSettings
      L2_4 = L2_4(L3_4, L4_4, L5_4, L6_4)
      L3_4 = "NEGATIVE LAB PRO v"
      L4_4 = _G
      L4_4 = L4_4.version
      L3_4 = L3_4 .. L4_4
      L4_4 = "Apply"
      L5_4 = NLPSessionStatus
      if L5_4 == 1 then
        L5_4 = "NEGATIVE LAB PRO v"
        L6_4 = _G
        L6_4 = L6_4.version
        L7_4 = " [trial mode]"
        L3_4 = L5_4 .. L6_4 .. L7_4
        L5_4 = "Apply ("
        L6_4 = allowance
        L7_4 = L13_1
        L7_4 = L7_4.photosAppliedV7
        L6_4 = L6_4 - L7_4
        L7_4 = " left)"
        L4_4 = L5_4 .. L6_4 .. L7_4
      end
      L5_4 = NLPSessionStatus
      if L5_4 == 2 then
        L4_4 = "Buy Negative Lab"
      end
      L5_4 = NLPSessionStatus
      if L5_4 == 2 then
        L5_4 = L5_1
        L5_4 = L5_4.confirm
        L6_4 = "Unlock Negative Lab Pro?"
        L7_4 = "Heyo! Looks like you're trying to convert "
        L8_4 = activePhotos
        L8_4 = #L8_4
        L9_4 = " negatives but you only have "
        L10_4 = allowance
        L11_4 = L13_1
        L11_4 = L11_4.photosAppliedV7
        L10_4 = L10_4 - L11_4
        L11_4 = [[
 free trials remaining. To continue using, you must purchase a license.

If you already have a license, go to 'File > Plugin Manager' to submit your license]]
        L7_4 = L7_4 .. L8_4 .. L9_4 .. L10_4 .. L11_4
        L8_4 = "Purchase License to Unlock"
        L5_4 = L5_4(L6_4, L7_4, L8_4)
        if L5_4 == "ok" then
          L6_4 = L11_1
          L6_4 = L6_4.openUrlInBrowser
          L7_4 = "https://www.negativelabpro.com/pricing/"
          L6_4(L7_4)
        end
      else
        L5_4 = L7_1
        L5_4 = L5_4.sleep
        L6_4 = 0
        L5_4(L6_4)
        L5_4 = L5_1
        L5_4 = L5_4.presentModalDialog
        L6_4 = {}
        L6_4.title = L3_4
        L6_4.contents = L1_4
        L6_4.accessoryView = L2_4
        L6_4.actionVerb = L4_4
        L6_4.save_frame = "NEGATIVEFramePosition"
        L7_4 = {}
        L8_4 = {}
        L9_4 = uiSettings
        L8_4.bind_to_object = L9_4
        L8_4.key = "isConverted"
        L7_4.enabled = L8_4
        L6_4.actionBinding = L7_4
        L5_4 = L5_4(L6_4)
        if L5_4 == "ok" then
          L6_4 = L10_1
          L6_4 = L6_4.stopTracking
          L6_4()
          L6_4 = NLPSessionStatus
          if L6_4 == 2 then
            L6_4 = L10_1
            L6_4 = L6_4.stopTracking
            L6_4()
            L6_4 = L24_1
            if L6_4 then
              L6_4 = 1
              L7_4 = undoCount
              L8_4 = 1
              for L9_4 = L6_4, L7_4, L8_4 do
                L10_4 = L12_1
                L10_4 = L10_4.undo
                L10_4()
                L10_4 = L7_1
                L10_4 = L10_4.sleep
                L11_4 = 0.1
                L10_4(L11_4)
              end
            end
            L6_4 = L19_1
            L6_4 = L6_4.restorePreviousSettings
            L6_4()
            L6_4 = L15_1
            L6_4 = L6_4.copyObservableTable
            L7_4 = properties
            L6_4 = L6_4(L7_4)
            L7_4 = L20_1
            L7_4 = L7_4.endSession
            L7_4()
            L7_4 = L11_1
            L7_4 = L7_4.openUrlInBrowser
            L8_4 = "http://gum.co/negative-lab"
            L7_4(L8_4)
          else
            L6_4 = L20_1
            L6_4 = L6_4.isConverted
            L7_4 = activePhotoIndex
            L6_4 = L6_4(L7_4)
            if L6_4 ~= true then
              L6_4 = L19_1
              L6_4 = L6_4.resetCrop
              L6_4()
            end
            L6_4 = NLPSessionStatus
            if L6_4 == 1 then
              L6_4 = L13_1
              L7_4 = L13_1
              L7_4 = L7_4.photosAppliedV7
              L8_4 = activePhotos
              L8_4 = #L8_4
              L7_4 = L7_4 + L8_4
              L6_4.photosAppliedV7 = L7_4
            end
            L6_4 = L20_1
            L6_4 = L6_4.updateActivePhotoMetadata
            L7_4 = properties
            L6_4(L7_4)
            L6_4 = L20_1
            L6_4 = L6_4.endSession
            L6_4()
            L6_4 = uiSettings
            L6_4 = L6_4.tiffCopy
            if L6_4 == true then
              L6_4 = L25_1
              L7_4 = L6_4
              L6_4 = L6_4.getTargetPhotos
              L6_4 = L6_4(L7_4)
              L7_4 = L23_1
              L8_4 = L7_4
              L7_4 = L7_4.trace
              L9_4 = "MAKE POSITIVE COPY of "
              L10_4 = #L6_4
              L11_4 = "photos"
              L9_4 = L9_4 .. L10_4 .. L11_4
              L7_4(L8_4, L9_4)
              L7_4 = L19_1
              L7_4 = L7_4.makePositiveCopy
              L8_4 = L6_4
              L9_4 = uiSettings
              L7_4(L8_4, L9_4)
            end
          end
        elseif L5_4 == "processing" then
        else
          L6_4 = L10_1
          L6_4 = L6_4.stopTracking
          L6_4()
          L6_4 = L7_1
          L6_4 = L6_4.startAsyncTask
          
          function L7_4()
            local L0_5, L1_5
            L0_5 = NLPSessionStatus
            if L0_5 ~= 1 then
              L0_5 = NLPSessionStatus
              if L0_5 ~= 2 then
                goto lbl_20
              end
            end
            L0_5 = L12_1
            L0_5 = L0_5.undo
            L0_5()
            L0_5 = WIN_ENV
            if L0_5 then
              L0_5 = L7_1
              L0_5 = L0_5.sleep
              L1_5 = 0.25
              L0_5(L1_5)
              L0_5 = L12_1
              L0_5 = L0_5.undo
              L0_5()
            end
            ::lbl_20::
            L0_5 = L19_1
            L0_5 = L0_5.restorePreviousSettings
            L0_5()
            L0_5 = L20_1
            L0_5 = L0_5.clearSession
            L0_5()
          end
          
          L6_4(L7_4)
        end
      end
    end
    
    L1_3(L2_3, L3_3)
  end
  
  L0_2(L1_2)
end

L26_1.mainDialog = L27_1
L26_1 = NLPDialogs
return L26_1
