local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1, L19_1, L20_1, L21_1, L22_1, L23_1, L24_1, L25_1, L26_1, L27_1, L28_1, L29_1, L30_1, L31_1, L32_1, L33_1, L34_1, L35_1, L36_1, L37_1

L1_1 = import
L2_1 = "LrApplication"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrApplicationView"
L2_1 = L2_1(L3_1)
L3_1 = import
L4_1 = "LrFunctionContext"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrDialogs"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrLogger"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrProgressScope"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrPathUtils"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrFileUtils"
L8_1 = L8_1(L9_1)
L9_1 = import
L10_1 = "LrTasks"
L9_1 = L9_1(L10_1)
L10_1 = import
L11_1 = "LrSystemInfo"
L10_1 = L10_1(L11_1)
L11_1 = import
L12_1 = "LrDevelopController"
L11_1 = L11_1(L12_1)
L12_1 = import
L13_1 = "LrPrefs"
L12_1 = L12_1(L13_1)
L12_1 = L12_1.prefsForPlugin
L12_1 = L12_1()
L13_1 = require
L14_1 = "UtilV3"
L13_1 = L13_1(L14_1)
L14_1 = require
L15_1 = "NegativePresets"
L14_1 = L14_1(L15_1)
L15_1 = require
L16_1 = "inspect"
L15_1 = L15_1(L16_1)
L16_1 = require
L17_1 = "NLPColorV3"
L16_1 = L16_1(L17_1)
L17_1 = require
L18_1 = "JSON"
L17_1 = L17_1(L18_1)
L18_1 = require
L19_1 = "hashV3"
L18_1 = L18_1(L19_1)
L19_1 = require
L20_1 = "History"
L19_1 = L19_1(L20_1)
L20_1 = L5_1
L21_1 = "NLPSession"
L20_1 = L20_1(L21_1)
L21_1 = _G
L21_1 = L21_1.dev
if L21_1 == true then
  L22_1 = L20_1
  L21_1 = L20_1.enable
  L23_1 = "logfile"
  L21_1(L22_1, L23_1)
else
  L22_1 = L20_1
  L21_1 = L20_1.disable
  L21_1(L22_1)
end
L22_1 = L20_1
L21_1 = L20_1.trace
L23_1 = "In NLP Session"
L21_1(L22_1, L23_1)
L21_1 = _G
L22_1 = {}
L21_1.activePhotos = L22_1
L21_1 = _G
L21_1.activePhoto = nil
L21_1 = _G
L21_1.activePhotoIndex = 1
L21_1 = _G
L22_1 = {}
L21_1.activePhotosMeta = L22_1
L21_1 = _G
L21_1.NLPSessionStatus = 0
L21_1 = _G
L22_1 = {}
L21_1.NLPDefaultSettings = L22_1
L21_1 = _G
L21_1.enhancedProfilesEnabled = false
L21_1 = _G
L21_1.enhancedProfilesDirectory = nil
L21_1 = _G
L21_1.enhancedProfilesErrorMsg = nil
L21_1 = _G
L21_1.enhancedProfileSaturation = true
L21_1 = _G
L21_1.enhancedProfileSaturationMessage = nil
L22_1 = L20_1
L21_1 = L20_1.trace
L23_1 = "SYSTEM ARCH: "
L24_1 = L10_1.architecture
L24_1 = L24_1()
L23_1 = L23_1 .. L24_1
L21_1(L22_1, L23_1)
L21_1 = _G
L22_1 = L10_1.architecture
L22_1 = L22_1()
L22_1 = L22_1 == "arm64"
L21_1.isAppleSilicon = L22_1
L21_1 = _G
L21_1.isPositive = false
L21_1 = L1_1.activeCatalog
L21_1 = L21_1()
L22_1 = L7_1.getStandardFilePath
L23_1 = "appData"
L22_1 = L22_1(L23_1)
L23_1 = L7_1.parent
L24_1 = L22_1
L23_1 = L23_1(L24_1)
L22_1 = L23_1
L23_1 = L7_1.parent
L24_1 = L22_1
L23_1 = L23_1(L24_1)
L22_1 = L23_1
L23_1 = L7_1.child
L24_1 = L22_1
L25_1 = "Negative Lab Pro"
L23_1 = L23_1(L24_1, L25_1)
L24_1 = L7_1.child
L25_1 = L23_1
L26_1 = "Rolls"
L24_1 = L24_1(L25_1, L26_1)
L25_1 = "rolls.json"
L26_1 = L7_1.child
L27_1 = L23_1
L28_1 = "Presets"
L26_1 = L26_1(L27_1, L28_1)
L27_1 = MAC_ENV
if L27_1 then
  L27_1 = "/"
  if L27_1 then
    goto lbl_151
  end
end
L27_1 = "\\"
::lbl_151::
L28_1 = L7_1.child
L29_1 = L23_1
L30_1 = "Backup"
L28_1 = L28_1(L29_1, L30_1)
L29_1 = "backup-license.txt"
L30_1 = {}
NLPSession = L30_1
L30_1 = NLPSession

function L31_1()
  local L0_2, L1_2
  L0_2 = activePhotosMeta
  return L0_2
end

L30_1.getActivePhotosMeta = L31_1
L30_1 = NLPSession

function L31_1()
  local L0_2, L1_2
  L0_2 = activePhotos
  return L0_2
end

L30_1.getActivePhotos = L31_1
L30_1 = NLPSession

function L31_1()
  local L0_2, L1_2
  L0_2 = sessionStatus
  return L0_2
end

L30_1.getSessionStatus = L31_1
L30_1 = NLPSession

function L31_1()
  local L0_2, L1_2
  L0_2 = _G
  L1_2 = {}
  L0_2.activePhotos = L1_2
  L0_2 = _G
  L0_2.activePhoto = nil
  L0_2 = _G
  L0_2.activePhotoIndex = 1
  L0_2 = _G
  L1_2 = {}
  L0_2.activePhotosMeta = L1_2
  L0_2 = _G
  L0_2.NLPSessionStatus = 0
  L0_2 = _G
  L1_2 = {}
  L0_2.NLPDefaultSettings = L1_2
  L0_2 = _G
  L0_2.enhancedProfilesEnabled = false
  L0_2 = _G
  L0_2.enhancedProfilesDirectory = nil
  L0_2 = _G
  L0_2.enhancedProfilesErrorMsg = nil
  L0_2 = _G
  L0_2.enhancedProfileSaturation = true
  L0_2 = _G
  L0_2.RollCache = nil
  L0_2 = _G
  L0_2.rolls = nil
  L0_2 = _G
  L0_2.nlpPresets = nil
  L0_2 = L19_1
  L0_2 = L0_2.clearHistory
  L0_2()
end

L30_1.clearSession = L31_1

function L30_1()
  local L0_2, L1_2, L2_2, L3_2
  L0_2 = L17_1
  
  function L1_2(A0_3, A1_3, A2_3, A3_3, A4_3)
  end
  
  L0_2.onDecodeError = L1_2
  L0_2 = L24_1
  L1_2 = L27_1
  L2_2 = L25_1
  L0_2 = L0_2 .. L1_2 .. L2_2
  rollPath = L0_2
  L0_2 = L8_1
  L0_2 = L0_2.exists
  L1_2 = rollPath
  L0_2 = L0_2(L1_2)
  if L0_2 then
    L0_2 = L8_1
    L0_2 = L0_2.readFile
    L1_2 = rollPath
    L0_2 = L0_2(L1_2)
    L1_2 = L17_1
    L2_2 = L1_2
    L1_2 = L1_2.decode
    L3_2 = L0_2
    L1_2 = L1_2(L2_2, L3_2)
    if L1_2 then
      L2_2 = _G
      L2_2.rolls = L1_2
    end
  else
    L0_2 = _G
    L1_2 = {}
    L0_2.rolls = L1_2
    L0_2 = _G
    L0_2 = L0_2.rolls
    L1_2 = {}
    L0_2.groups = L1_2
  end
  L0_2 = _G
  L0_2 = L0_2.activePhotosMeta
  L1_2 = activePhotoIndex
  L0_2 = L0_2[L1_2]
  L0_2 = L0_2.roll
  if L0_2 ~= nil then
    L0_2 = "rollID_"
    L1_2 = _G
    L1_2 = L1_2.activePhotosMeta
    L2_2 = activePhotoIndex
    L1_2 = L1_2[L2_2]
    L1_2 = L1_2.roll
    L1_2 = L1_2.ID
    L0_2 = L0_2 .. L1_2
    L1_2 = _G
    L1_2 = L1_2.rolls
    L1_2 = L1_2[L0_2]
    if L1_2 ~= nil then
      L1_2 = _G
      L1_2 = L1_2.activePhotosMeta
      L2_2 = activePhotoIndex
      L1_2 = L1_2[L2_2]
      L1_2 = L1_2.roll
      L2_2 = _G
      L2_2 = L2_2.rolls
      L2_2 = L2_2[L0_2]
      L2_2 = L2_2.rollName
      L1_2.rollName = L2_2
    else
      L1_2 = _G
      L1_2 = L1_2.rolls
      L2_2 = _G
      L2_2 = L2_2.activePhotosMeta
      L3_2 = activePhotoIndex
      L2_2 = L2_2[L3_2]
      L2_2 = L2_2.roll
      L1_2[L0_2] = L2_2
    end
  end
end

loadRollFile = L30_1

function L30_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L0_2 = _G
  L1_2 = {}
  L0_2.nlpPresets = L1_2
  L0_2 = L8_1
  L0_2 = L0_2.exists
  L1_2 = L26_1
  L0_2 = L0_2(L1_2)
  if not L0_2 then
    L0_2 = L8_1
    L0_2 = L0_2.createAllDirectories
    L1_2 = L26_1
    L0_2(L1_2)
  else
    L0_2 = L8_1
    L0_2 = L0_2.recursiveFiles
    L1_2 = L26_1
    L0_2, L1_2, L2_2 = L0_2(L1_2)
    for L3_2 in L0_2, L1_2, L2_2 do
      L4_2 = L20_1
      L5_2 = L4_2
      L4_2 = L4_2.trace
      L6_2 = L3_2
      L4_2(L5_2, L6_2)
      L4_2 = L7_1
      L4_2 = L4_2.extension
      L5_2 = L3_2
      L4_2 = L4_2(L5_2)
      if L4_2 == "json" then
        L4_2 = L8_1
        L4_2 = L4_2.readFile
        L5_2 = L3_2
        L4_2 = L4_2(L5_2)
        L5_2 = L17_1
        L6_2 = L5_2
        L5_2 = L5_2.decode
        L7_2 = L4_2
        L5_2, L6_2 = L5_2(L6_2, L7_2)
        L7_2 = L7_1
        L7_2 = L7_2.makeRelative
        L8_2 = L3_2
        L9_2 = L26_1
        L7_2 = L7_2(L8_2, L9_2)
        if L5_2 ~= nil then
          L8_2 = L5_2.id
          if L8_2 ~= nil then
            L5_2.fileName = L7_2
            L5_2.needsUpdate = false
            L8_2 = L5_2.isPositive
            if L8_2 == true then
              L8_2 = _G
              L8_2 = L8_2.isPositive
              if L8_2 == true then
                L8_2 = _G
                L8_2 = L8_2.nlpPresets
                L9_2 = L5_2.id
                L8_2[L9_2] = L5_2
            end
            else
              L8_2 = L5_2.isMonochrome
              if L8_2 == true then
                L8_2 = _G
                L8_2 = L8_2.isMonochrome
                if L8_2 == true then
                  L8_2 = _G
                  L8_2 = L8_2.nlpPresets
                  L9_2 = L5_2.id
                  L8_2[L9_2] = L5_2
              end
              else
                L8_2 = L5_2.isMonochrome
                if L8_2 ~= true then
                  L8_2 = _G
                  L8_2 = L8_2.isMonochrome
                  if L8_2 ~= true then
                    L8_2 = _G
                    L8_2 = L8_2.nlpPresets
                    L9_2 = L5_2.id
                    L8_2[L9_2] = L5_2
                  end
                end
              end
            end
          end
        end
      end
    end
  end
end

loadPresetFiles = L30_1

function L30_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L0_2 = pairs
  L1_2 = _G
  L1_2 = L1_2.nlpPresets
  L0_2, L1_2, L2_2 = L0_2(L1_2)
  for L3_2, L4_2 in L0_2, L1_2, L2_2 do
    L5_2 = L4_2.needsUpdate
    if L5_2 == true then
      L5_2 = _G
      L5_2 = L5_2.nlpPresets
      L5_2 = L5_2[L3_2]
      L5_2.needsUpdate = nil
      L5_2 = L17_1
      L6_2 = L5_2
      L5_2 = L5_2.encode_pretty
      L7_2 = _G
      L7_2 = L7_2.nlpPresets
      L7_2 = L7_2[L3_2]
      L5_2 = L5_2(L6_2, L7_2)
      L6_2 = io
      L6_2 = L6_2.open
      L7_2 = L26_1
      L8_2 = L27_1
      L9_2 = L4_2.fileName
      L7_2 = L7_2 .. L8_2 .. L9_2
      L8_2 = "w"
      L6_2 = L6_2(L7_2, L8_2)
      L8_2 = L6_2
      L7_2 = L6_2.write
      L9_2 = L5_2
      L7_2(L8_2, L9_2)
      L8_2 = L6_2
      L7_2 = L6_2.close
      L7_2(L8_2)
    end
  end
  L0_2 = _G
  L0_2 = L0_2.presetDeleteList
  if L0_2 then
    L0_2 = 1
    L1_2 = _G
    L1_2 = L1_2.presetDeleteList
    L1_2 = #L1_2
    L2_2 = 1
    for L3_2 = L0_2, L1_2, L2_2 do
      L4_2 = L26_1
      L5_2 = L27_1
      L6_2 = _G
      L6_2 = L6_2.presetDeleteList
      L6_2 = L6_2[L3_2]
      L4_2 = L4_2 .. L5_2 .. L6_2
      L5_2 = L8_1
      L5_2 = L5_2.exists
      L6_2 = L4_2
      L5_2 = L5_2(L6_2)
      if L5_2 then
        L5_2 = L8_1
        L5_2 = L5_2.delete
        L6_2 = L4_2
        L5_2(L6_2)
      end
    end
  end
end

L31_1 = NLPSession

function L32_1()
  local L0_2, L1_2
  L0_2 = _G
  L0_2 = L0_2.activePhotosMeta
  L1_2 = _G
  L1_2 = L1_2.activePhotoIndex
  L0_2 = L0_2[L1_2]
  L1_2 = L0_2.settings
  if L1_2 ~= nil then
    L1_2 = L0_2.settings
    L1_2 = L1_2.brightness
    if L1_2 ~= nil then
      L1_2 = _G
      L1_2.hasSettings = true
  end
  else
    L1_2 = _G
    L1_2.hasSettings = false
  end
end

L31_1.setHasSettings = L32_1
L31_1 = NLPSession

function L32_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2
  L0_2 = _G
  L0_2 = L0_2.activePhotosMeta
  if L0_2 ~= nil then
    L0_2 = _G
    L0_2 = L0_2.activePhotoIndex
    if L0_2 ~= nil then
      L0_2 = _G
      L0_2 = L0_2.activePhotosMeta
      L1_2 = _G
      L1_2 = L1_2.activePhotoIndex
      L0_2 = L0_2[L1_2]
      if L0_2 ~= nil then
        L0_2 = _G
        L0_2 = L0_2.activePhotosMeta
        L1_2 = _G
        L1_2 = L1_2.activePhotoIndex
        L0_2 = L0_2[L1_2]
        L0_2 = L0_2.settings
        if L0_2 ~= nil then
          L0_2 = _G
          L0_2 = L0_2.activePhotosMeta
          L1_2 = _G
          L1_2 = L1_2.activePhotoIndex
          L0_2 = L0_2[L1_2]
          L0_2 = L0_2.settings
          L0_2 = L0_2.colorModel
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.activePhotosMeta
            L1_2 = _G
            L1_2 = L1_2.activePhotoIndex
            L0_2 = L0_2[L1_2]
            L0_2 = L0_2.settings
            L0_2 = L0_2.colorModel
            L1_2 = _G
            L1_2 = L1_2.activePhotosMeta
            L2_2 = _G
            L2_2 = L2_2.activePhotoIndex
            L1_2 = L1_2[L2_2]
            L1_2 = L1_2.settings
            L1_2 = L1_2.imageType
            if L0_2 == "blackandwhite" then
              L2_2 = _G
              L2_2.isMonochrome = true
            else
              L2_2 = _G
              L2_2.isMonochrome = false
            end
            if L1_2 == "editOnly" or L1_2 == "positive" then
              L2_2 = _G
              L2_2.isPositive = true
              L2_2 = L20_1
              L3_2 = L2_2
              L2_2 = L2_2.trace
              L4_2 = "isPositive is true"
              L2_2(L3_2, L4_2)
            else
              L2_2 = _G
              L2_2.isPositive = false
              L2_2 = L20_1
              L3_2 = L2_2
              L2_2 = L2_2.trace
              L4_2 = "isPositive is false"
              L2_2(L3_2, L4_2)
            end
          end
        end
      end
    end
  end
end

L31_1.setIsMonochrome = L32_1

function L31_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2
  L0_2 = import
  L1_2 = "LrPathUtils"
  L0_2 = L0_2(L1_2)
  L1_2 = import
  L2_2 = "LrFileUtils"
  L1_2 = L1_2(L2_2)
  L2_2 = nil
  L3_2 = _PLUGIN
  L3_2 = L3_2.path
  L4_2 = MAC_ENV
  if L4_2 then
    L4_2 = "/license.txt"
    if L4_2 then
      goto lbl_17
    end
  end
  L4_2 = "\\license.txt"
  ::lbl_17::
  L3_2 = L3_2 .. L4_2
  L4_2 = L1_2.isReadable
  L5_2 = L3_2
  L4_2 = L4_2(L5_2)
  if L4_2 then
    L4_2 = L1_2.readFile
    L5_2 = L3_2
    L4_2 = L4_2(L5_2)
    L5_2 = L13_1
    L5_2 = L5_2.lines
    L6_2 = L4_2
    L5_2 = L5_2(L6_2)
    L6_2 = #L5_2
    if 2 < L6_2 then
      L2_2 = L5_2
    else
    end
  end
  return L2_2
end

function L32_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2
  L0_2 = L31_1
  L0_2 = L0_2()
  if L0_2 then
    L1_2 = L0_2[1]
    L2_2 = L0_2[2]
    L3_2 = L0_2[3]
    L4_2 = L18_1
    L4_2 = L4_2.blake3
    L5_2 = L1_1
    L5_2 = L5_2.macAddressHash
    L5_2 = L5_2()
    L6_2 = L4_2
    L7_2 = L5_2
    L9_2 = L1_2
    L8_2 = L1_2.sub
    L10_2 = 1
    L11_2 = 5
    L8_2, L9_2, L10_2, L11_2, L12_2 = L8_2(L9_2, L10_2, L11_2)
    L6_2 = L6_2(L7_2, L8_2, L9_2, L10_2, L11_2, L12_2)
    L7_2 = L4_2
    L8_2 = L6_2
    L10_2 = L2_2
    L9_2 = L2_2.sub
    L11_2 = 1
    L12_2 = 5
    L9_2, L10_2, L11_2, L12_2 = L9_2(L10_2, L11_2, L12_2)
    L7_2 = L7_2(L8_2, L9_2, L10_2, L11_2, L12_2)
    hashcheck = L7_2
    if L6_2 == L3_2 then
      L7_2 = _G
      L7_2.NLPSessionStatus = 3
    else
      L7_2 = _G
      L7_2.NLPSessionStatus = 2
    end
  else
    L1_2 = _G
    L1_2.NLPSessionStatus = 2
  end
end

L33_1 = NLPSession

function L34_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L2_2 = pairs
  L3_2 = _G
  L3_2 = L3_2.activePhotosMeta
  L3_2 = L3_2[A1_2]
  L3_2 = L3_2.settings
  L2_2, L3_2, L4_2 = L2_2(L3_2)
  for L5_2, L6_2 in L2_2, L3_2, L4_2 do
    if L5_2 == "colorPicker" then
    else
      A0_2[L5_2] = L6_2
    end
  end
end

L33_1.updatePropertiesFromMeta = L34_1
L33_1 = NLPSession

function L34_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L13_1
  L1_2 = L1_2.copyObservableTable
  L2_2 = A0_2
  L1_2 = L1_2(L2_2)
  L2_2 = _G
  L2_2 = L2_2.activePhotosMeta
  L3_2 = _G
  L3_2 = L3_2.activePhotoIndex
  L2_2 = L2_2[L3_2]
  L2_2.settings = L1_2
end

L33_1.updateActivePhotoMetadata = L34_1

function L33_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L1_2 = A0_2.film
  if L1_2 == "custom" then
    L1_2 = A0_2.hueStrength
    if L1_2 ~= 0 then
      L1_2 = nil
      L2_2 = nil
      L3_2 = nil
      L1_2 = A0_2.filmHue
      L2_2 = A0_2.hueStrength
      L1_2 = L1_2 + 180
      if L1_2 < 0 then
        L1_2 = L1_2 + 360
      end
      L4_2 = tonumber
      L5_2 = L2_2
      L4_2 = L4_2(L5_2)
      L2_2 = L4_2
      if L2_2 < 0 then
        L1_2 = L1_2 + 180
        if 360 <= L1_2 then
          L1_2 = L1_2 - 360
        end
        L4_2 = math
        L4_2 = L4_2.abs
        L5_2 = L2_2
        L4_2 = L4_2(L5_2)
        L2_2 = L4_2
      end
      L4_2 = L16_1
      L4_2 = L4_2.fromHSV
      L5_2 = L1_2
      L6_2 = L2_2 / 100
      L7_2 = 0.5
      L4_2 = L4_2(L5_2, L6_2, L7_2)
      L6_2 = L4_2
      L5_2 = L4_2.linearBalance
      L7_2 = "r"
      L5_2 = L5_2(L6_2, L7_2)
      L6_2 = L5_2[2]
      L6_2 = -L6_2
      L7_2 = _G
      L7_2 = L7_2.tempMultiplier
      L6_2 = L6_2 / L7_2
      L7_2 = L5_2[3]
      L7_2 = -L7_2
      L8_2 = _G
      L8_2 = L8_2.tempMultiplier
      L7_2 = L7_2 / L8_2
      L8_2 = L7_2
      L9_2 = L6_2
      return L8_2, L9_2
    else
    end
  end
end

updateHue = L33_1
L33_1 = NLPSession

function L34_1()
  local L0_2, L1_2
  L0_2 = L2_1
  L0_2 = L0_2.getCurrentModuleName
  L0_2 = L0_2()
  if L0_2 == "library" then
    L0_2 = L2_1
    L0_2 = L0_2.switchToModule
    L1_2 = "develop"
    L0_2(L1_2)
  end
end

L33_1.switchToDevelopModule = L34_1
L33_1 = NLPSession

function L34_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2
  L0_2 = L2_1
  L0_2 = L0_2.getCurrentModuleName
  L0_2 = L0_2()
  if L0_2 ~= "develop" then
    L0_2 = L2_1
    L0_2 = L0_2.getCurrentModuleName
    L0_2 = L0_2()
    if L0_2 ~= "library" then
      goto lbl_37
    end
  end
  L0_2 = _G
  L1_2 = L21_1
  L1_2 = L1_2.targetPhotos
  L0_2.activePhotos = L1_2
  L0_2 = _G
  L1_2 = L21_1
  L1_2 = L1_2.targetPhoto
  L0_2.activePhoto = L1_2
  L0_2 = 1
  L1_2 = _G
  L1_2 = L1_2.activePhotos
  L1_2 = #L1_2
  L2_2 = 1
  for L3_2 = L0_2, L1_2, L2_2 do
    L4_2 = _G
    L4_2 = L4_2.activePhotos
    L4_2 = L4_2[L3_2]
    L4_2 = L4_2.localIdentifier
    L5_2 = L21_1
    L5_2 = L5_2.targetPhoto
    L5_2 = L5_2.localIdentifier
    if L4_2 == L5_2 then
      L4_2 = _G
      L4_2.activePhotoIndex = L3_2
    end
  end
  ::lbl_37::
end

L33_1.setActivePhotos = L34_1
L33_1 = NLPSession

function L34_1(A0_2)
  local L1_2
  L1_2 = _G
  L1_2.activePhotoIndex = A0_2
end

L33_1.updateActivePhotoIndex = L34_1
L33_1 = NLPSession

function L34_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2
  L0_2 = L12_1
  L0_2 = L0_2.conversionSettings
  if L0_2 ~= nil then
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.source
    L0_2.source = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.filmBorder
    L0_2.filmBorder = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.saturation
    L0_2.saturation = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.colorModel
    L0_2.colorModel = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.previewBuffer
    L0_2.previewBuffer = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.convertAsRoll
    if not L1_2 then
      L1_2 = false
    end
    L0_2.convertAsRoll = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L12_1
    L1_2 = L1_2.conversionSettings
    L1_2 = L1_2.imageType
    if not L1_2 then
      L1_2 = L14_1
      L1_2 = L1_2.defaultSettings
      L1_2 = L1_2.imageType
    end
    L0_2.imageType = L1_2
  else
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.source
    L0_2.source = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.filmBorder
    L0_2.filmBorder = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.saturation
    L0_2.saturation = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.colorModel
    L0_2.colorModel = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.previewBuffer
    L0_2.previewBuffer = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.convertAsRoll
    L0_2.convertAsRoll = L1_2
    L0_2 = _G
    L0_2 = L0_2.NLPDefaultSettings
    L1_2 = L14_1
    L1_2 = L1_2.defaultSettings
    L1_2 = L1_2.imageType
    L0_2.imageType = L1_2
  end
  L0_2 = _G
  L0_2 = L0_2.isConverted
  if L0_2 == true then
  end
  L0_2 = _G
  L0_2 = L0_2.hasSettings
  if L0_2 == true then
  end
  L0_2 = _G
  L0_2 = L0_2.isConverted
  if L0_2 == true then
    L0_2 = _G
    L0_2 = L0_2.hasSettings
    if L0_2 == true then
      L0_2 = isPositive
      if L0_2 == true then
        L0_2 = pairs
        L1_2 = L14_1
        L1_2 = L1_2.defaultPositiveSettings
        L0_2, L1_2, L2_2 = L0_2(L1_2)
        for L3_2, L4_2 in L0_2, L1_2, L2_2 do
          L5_2 = _G
          L5_2 = L5_2.NLPDefaultSettings
          L5_2[L3_2] = L4_2
        end
      else
        L0_2 = isMonochrome
        if L0_2 == true then
          L0_2 = pairs
          L1_2 = L14_1
          L1_2 = L1_2.defaultMonoSettings
          L0_2, L1_2, L2_2 = L0_2(L1_2)
          for L3_2, L4_2 in L0_2, L1_2, L2_2 do
            L5_2 = _G
            L5_2 = L5_2.NLPDefaultSettings
            L5_2[L3_2] = L4_2
          end
        else
          L0_2 = pairs
          L1_2 = L14_1
          L1_2 = L1_2.defaultSettings
          L0_2, L1_2, L2_2 = L0_2(L1_2)
          for L3_2, L4_2 in L0_2, L1_2, L2_2 do
            L5_2 = _G
            L5_2 = L5_2.NLPDefaultSettings
            L5_2[L3_2] = L4_2
          end
        end
      end
      L0_2 = _G
      L0_2 = L0_2.NLPDefaultSettings
      L0_2.selectedPreset = "custom"
    end
  end
  L0_2 = _G
  L0_2 = L0_2.isConverted
  if L0_2 == true then
    L0_2 = _G
    L0_2 = L0_2.hasSettings
    if L0_2 ~= true then
      L0_2 = isPositive
      if L0_2 == true then
        L0_2 = pairs
        L1_2 = L14_1
        L1_2 = L1_2.defaultPositiveSettings
        L0_2, L1_2, L2_2 = L0_2(L1_2)
        for L3_2, L4_2 in L0_2, L1_2, L2_2 do
          L5_2 = _G
          L5_2 = L5_2.NLPDefaultSettings
          L5_2[L3_2] = L4_2
        end
        L0_2 = L12_1
        L0_2 = L0_2.defaultPresetPositive
        if L0_2 ~= nil then
          L0_2 = _G
          L0_2 = L0_2.nlpPresets
          L1_2 = L12_1
          L1_2 = L1_2.defaultPresetPositive
          L0_2 = L0_2[L1_2]
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.nlpPresets
            L1_2 = L12_1
            L1_2 = L1_2.defaultPresetPositive
            L0_2 = L0_2[L1_2]
            L0_2 = L0_2.settings
            L1_2 = pairs
            L2_2 = L0_2
            L1_2, L2_2, L3_2 = L1_2(L2_2)
            for L4_2, L5_2 in L1_2, L2_2, L3_2 do
              L6_2 = L13_1
              L6_2 = L6_2.keyIsInTable
              L7_2 = L14_1
              L7_2 = L7_2.availableSettings
              L8_2 = L4_2
              L6_2 = L6_2(L7_2, L8_2)
              if L6_2 then
                L6_2 = L14_1
                L6_2 = L6_2.availableSettings
                L6_2 = L6_2[L4_2]
                L6_2 = L6_2.internalName
                L7_2 = _G
                L7_2 = L7_2.NLPDefaultSettings
                L7_2[L6_2] = L5_2
              end
            end
            L1_2 = _G
            L1_2 = L1_2.NLPDefaultSettings
            L2_2 = L12_1
            L2_2 = L2_2.defaultPresetPositive
            L1_2.selectedPreset = L2_2
          end
        end
        L0_2 = L12_1
        L0_2 = L0_2.defaultPresetPositive
        if L0_2 ~= nil then
          L0_2 = _G
          L0_2 = L0_2.nlpPresets
          L1_2 = L12_1
          L1_2 = L1_2.defaultPresetPositive
          L0_2 = L0_2[L1_2]
          if L0_2 == nil then
            L0_2 = _G
            L0_2 = L0_2.NLPDefaultSettings
            L0_2.selectedPreset = "custom"
          end
        end
      else
        L0_2 = isMonochrome
        if L0_2 == true then
          L0_2 = pairs
          L1_2 = L14_1
          L1_2 = L1_2.defaultMonoSettings
          L0_2, L1_2, L2_2 = L0_2(L1_2)
          for L3_2, L4_2 in L0_2, L1_2, L2_2 do
            L5_2 = _G
            L5_2 = L5_2.NLPDefaultSettings
            L5_2[L3_2] = L4_2
          end
          L0_2 = L12_1
          L0_2 = L0_2.defaultPresetMono
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.nlpPresets
            L1_2 = L12_1
            L1_2 = L1_2.defaultPresetMono
            L0_2 = L0_2[L1_2]
            if L0_2 ~= nil then
              L0_2 = _G
              L0_2 = L0_2.nlpPresets
              L1_2 = L12_1
              L1_2 = L1_2.defaultPresetMono
              L0_2 = L0_2[L1_2]
              L0_2 = L0_2.settings
              L1_2 = pairs
              L2_2 = L0_2
              L1_2, L2_2, L3_2 = L1_2(L2_2)
              for L4_2, L5_2 in L1_2, L2_2, L3_2 do
                L6_2 = L13_1
                L6_2 = L6_2.keyIsInTable
                L7_2 = L14_1
                L7_2 = L7_2.availableSettings
                L8_2 = L4_2
                L6_2 = L6_2(L7_2, L8_2)
                if L6_2 then
                  L6_2 = L14_1
                  L6_2 = L6_2.availableSettings
                  L6_2 = L6_2[L4_2]
                  L6_2 = L6_2.internalName
                  L7_2 = _G
                  L7_2 = L7_2.NLPDefaultSettings
                  L7_2[L6_2] = L5_2
                end
              end
              L1_2 = _G
              L1_2 = L1_2.NLPDefaultSettings
              L2_2 = L12_1
              L2_2 = L2_2.defaultPresetMono
              L1_2.selectedPreset = L2_2
            end
          end
          L0_2 = L12_1
          L0_2 = L0_2.defaultPresetMono
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.nlpPresets
            L1_2 = L12_1
            L1_2 = L1_2.defaultPresetMono
            L0_2 = L0_2[L1_2]
            if L0_2 == nil then
              L0_2 = _G
              L0_2 = L0_2.NLPDefaultSettings
              L0_2.selectedPreset = "custom"
            end
          end
        else
          L0_2 = pairs
          L1_2 = L14_1
          L1_2 = L1_2.defaultSettings
          L0_2, L1_2, L2_2 = L0_2(L1_2)
          for L3_2, L4_2 in L0_2, L1_2, L2_2 do
            L5_2 = _G
            L5_2 = L5_2.NLPDefaultSettings
            L5_2[L3_2] = L4_2
          end
          L0_2 = L12_1
          L0_2 = L0_2.defaultPresetColor
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.nlpPresets
            L1_2 = L12_1
            L1_2 = L1_2.defaultPresetColor
            L0_2 = L0_2[L1_2]
            if L0_2 ~= nil then
              L0_2 = _G
              L0_2 = L0_2.nlpPresets
              L1_2 = L12_1
              L1_2 = L1_2.defaultPresetColor
              L0_2 = L0_2[L1_2]
              L0_2 = L0_2.settings
              L1_2 = pairs
              L2_2 = L0_2
              L1_2, L2_2, L3_2 = L1_2(L2_2)
              for L4_2, L5_2 in L1_2, L2_2, L3_2 do
                L6_2 = L13_1
                L6_2 = L6_2.keyIsInTable
                L7_2 = L14_1
                L7_2 = L7_2.availableSettings
                L8_2 = L4_2
                L6_2 = L6_2(L7_2, L8_2)
                if L6_2 then
                  L6_2 = L14_1
                  L6_2 = L6_2.availableSettings
                  L6_2 = L6_2[L4_2]
                  L6_2 = L6_2.internalName
                  L7_2 = _G
                  L7_2 = L7_2.NLPDefaultSettings
                  L7_2[L6_2] = L5_2
                end
              end
              L1_2 = _G
              L1_2 = L1_2.NLPDefaultSettings
              L2_2 = L12_1
              L2_2 = L2_2.defaultPresetColor
              L1_2.selectedPreset = L2_2
            end
          end
          L0_2 = L12_1
          L0_2 = L0_2.defaultPresetColor
          if L0_2 ~= nil then
            L0_2 = _G
            L0_2 = L0_2.nlpPresets
            L1_2 = L12_1
            L1_2 = L1_2.defaultPresetColor
            L0_2 = L0_2[L1_2]
            if L0_2 == nil then
              L0_2 = _G
              L0_2 = L0_2.NLPDefaultSettings
              L0_2.selectedPreset = "custom"
            end
          end
        end
      end
    end
  end
end

L33_1.buildDefaultSettings = L34_1
L33_1 = NLPSession

function L34_1()
  local L0_2, L1_2
  L0_2 = L2_1
  L0_2 = L0_2.getCurrentModuleName
  L0_2 = L0_2()
  if L0_2 == "library" then
    L0_2 = _G
    L0_2 = L0_2.activePhotos
    L0_2 = #L0_2
    if 1 < L0_2 then
      L0_2 = true
      if L0_2 then
        goto lbl_15
      end
    end
  end
  L0_2 = false
  ::lbl_15::
  return L0_2
end

L33_1.isBatchMode = L34_1
L33_1 = NLPSession

function L34_1()
  local L0_2, L1_2
  L0_2 = L1_1
  L0_2 = L0_2.versionTable
  L0_2 = L0_2()
  L1_2 = L0_2.major
  L1_2 = 13 <= L1_2
  IsGroupMaskAvailable = L1_2
  L1_2 = IsGroupMaskAvailable
  return L1_2
end

L33_1.isGroupMaskAvailable = L34_1

function L33_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2
  L1_2 = _G
  L1_2 = L1_2.activePhotosMeta
  L1_2 = L1_2[A0_2]
  L1_2 = L1_2.previousLightroomSettings
  L1_2 = L1_2.MaskGroupBasedCorrections
  if L1_2 ~= nil then
    L1_2 = _G
    L1_2 = L1_2.activePhotosMeta
    L1_2 = L1_2[A0_2]
    L2_2 = L13_1
    L2_2 = L2_2.deepCopy
    L3_2 = _G
    L3_2 = L3_2.activePhotosMeta
    L3_2 = L3_2[A0_2]
    L3_2 = L3_2.previousLightroomSettings
    L3_2 = L3_2.MaskGroupBasedCorrections
    L2_2 = L2_2(L3_2)
    L1_2.activeMasks = L2_2
    L1_2 = 1
    L2_2 = _G
    L2_2 = L2_2.activePhotosMeta
    L2_2 = L2_2[A0_2]
    L2_2 = L2_2.activeMasks
    L2_2 = #L2_2
    L3_2 = 1
    for L4_2 = L1_2, L2_2, L3_2 do
      L5_2 = _G
      L5_2 = L5_2.activePhotosMeta
      L5_2 = L5_2[A0_2]
      L5_2 = L5_2.activeMasks
      L5_2 = L5_2[L4_2]
      L5_2 = L5_2.CorrectionName
      if L5_2 == "Negative Lab Pro" then
        L5_2 = L20_1
        L6_2 = L5_2
        L5_2 = L5_2.trace
        L7_2 = "Negative Lab Pro Correction found on layer "
        L8_2 = L4_2
        L9_2 = " of photo "
        L10_2 = A0_2
        L7_2 = L7_2 .. L8_2 .. L9_2 .. L10_2
        L5_2(L6_2, L7_2)
        L5_2 = _G
        L5_2 = L5_2.activePhotosMeta
        L5_2 = L5_2[A0_2]
        L5_2.existingNLPMaskIndex = L4_2
      end
    end
  else
    L1_2 = _G
    L1_2 = L1_2.activePhotosMeta
    L1_2 = L1_2[A0_2]
    L1_2 = L1_2.previousLightroomSettings
    L2_2 = {}
    L1_2.MaskGroupBasedCorrections = L2_2
  end
end

L34_1 = NLPSession

function L35_1(A0_2)
  local L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L4_2 = L1_1
  L4_2 = L4_2.versionTable
  L4_2 = L4_2()
  L5_2 = nil
  L6_2 = nil
  L7_2 = L1_1
  L7_2 = L7_2.developPresetFolders
  L7_2 = L7_2()
  L5_2 = L7_2
  L7_2 = 1
  L8_2 = #L5_2
  L9_2 = 1
  for L10_2 = L7_2, L8_2, L9_2 do
    L11_2 = L5_2[L10_2]
    L12_2 = L11_2
    L11_2 = L11_2.getName
    L11_2 = L11_2(L12_2)
    if L11_2 == "NLP Enhanced Settings" then
      L6_2 = L10_2
    end
  end
  L7_2 = L4_2.major
  L7_2 = 7 < L7_2
  L8_2 = L6_2 ~= nil
  L9_2 = A0_2.colorModel
  L9_2 = L9_2 ~= "none"
  L1_2 = L7_2 or L1_2
  L1_2 = L8_2 or L1_2
  if L7_2 and L8_2 then
    L1_2 = L9_2
  end
  L10_2 = true
  L11_2 = nil
  L12_2 = scanSource
  if L12_2 == "linearScan" then
    L12_2 = "Tif"
    if L12_2 then
      goto lbl_69
    end
  end
  L12_2 = scanSource
  if L12_2 == "vuescanRaw" then
    L12_2 = "RawScan"
    if L12_2 then
      goto lbl_69
    end
  end
  L12_2 = scanSource
  L12_2 = L12_2 == "cameraScan" and L12_2
  ::lbl_69::
  scannerType = L12_2
  L12_2 = _G
  L12_2 = L12_2.activePhotosMeta
  L13_2 = _G
  L13_2 = L13_2.activePhotoIndex
  L12_2 = L12_2[L13_2]
  L12_2 = L12_2.nlpVersion
  if L12_2 == "2.2.0" then
    L13_2 = scannerType
    if L13_2 ~= "Tif" then
      goto lbl_101
    end
  end
  if L12_2 == "2.2" then
    L13_2 = scannerType
    if L13_2 ~= "Tif" then
      goto lbl_101
    end
  end
  if L12_2 == "2.1.2" then
    L13_2 = scannerType
    if L13_2 ~= "Tif" then
      goto lbl_101
    end
  end
  if L12_2 == "2.1" then
    L13_2 = scannerType
    if L13_2 ~= "Tif" then
      goto lbl_101
    end
  end
  if L12_2 == "2.0" then
    L13_2 = scannerType
    ::lbl_101::
    if L13_2 ~= "Tif" then
      L10_2 = false
    end
  end
  if L1_2 then
    L13_2 = L5_2[L6_2]
    L14_2 = L13_2
    L13_2 = L13_2.getDevelopPresets
    L13_2 = L13_2(L14_2)
    L2_2 = L13_2
  end
  L3_2 = false
  if not L7_2 then
    L13_2 = "LUTs require LrC 7.3 or later"
    errrorMsg = L13_2
  elseif not L8_2 then
    L3_2 = "Could not locate LUTs"
  elseif not L9_2 then
    L13_2 = A0_2.colorModel
    if L13_2 == "blackandwhite" then
      L3_2 = "LUTs won't work with 'b+w'"
    else
      L13_2 = A0_2.colorModel
      if L13_2 == "none" then
        L3_2 = "LUTs require a color model"
      end
    end
  end
  L13_2 = _G
  L13_2.enhancedProfilesEnabled = L1_2
  L13_2 = _G
  L13_2.enhancedProfilesDirectory = L2_2
  L13_2 = _G
  L13_2.enhancedProfilesErrorMsg = L3_2
  L13_2 = _G
  L13_2.enhancedProfileSaturation = L10_2
  if L1_2 then
  else
  end
  L13_2 = L1_2
  L14_2 = L2_2
  L15_2 = L3_2
  L16_2 = L10_2
  return L13_2, L14_2, L15_2, L16_2
end

L34_1.useEnhancedProfiles = L35_1
L34_1 = NLPSession

function L35_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2
  L0_2 = {}
  L1_2 = L28_1
  L2_2 = L27_1
  L3_2 = L29_1
  L1_2 = L1_2 .. L2_2 .. L3_2
  L2_2 = L8_1
  L2_2 = L2_2.exists
  L3_2 = L1_2
  L2_2 = L2_2(L3_2)
  if L2_2 then
    L2_2 = L8_1
    L2_2 = L2_2.readFile
    L3_2 = L1_2
    L2_2 = L2_2(L3_2)
    L3_2 = L13_1
    L3_2 = L3_2.lines
    L4_2 = L2_2
    L3_2 = L3_2(L4_2)
    L4_2 = #L3_2
    if L4_2 == 4 then
      L4_2 = L3_2[1]
      L0_2.email_address = L4_2
      L4_2 = L3_2[2]
      L0_2.license_key = L4_2
      L4_2 = L3_2[3]
      L0_2.instance_id = L4_2
      L4_2 = L3_2[4]
      L0_2.hash_table = L4_2
      return L0_2
    end
  end
  L2_2 = nil
  return L2_2
end

L34_1.getBackupLicense = L35_1
L34_1 = NLPSession

function L35_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2
  L0_2 = _G
  L0_2 = L0_2.activePhotos
  L0_2 = #L0_2
  L1_2 = true
  L2_2 = L12_1
  L3_2 = L12_1
  L3_2 = L3_2.photosAppliedV7
  if not L3_2 then
    L3_2 = 0
  end
  L2_2.photosAppliedV7 = L3_2
  L2_2 = L12_1
  L2_2 = L2_2.photosAppliedV7
  L3_2 = 24
  allowance = L3_2
  L3_2 = allowance
  L3_2 = L3_2 - L2_2
  L4_2 = L13_1
  L4_2 = L4_2.v3
  L4_2 = L4_2()
  L5_2 = "-k"
  L4_2 = L4_2 .. L5_2
  L5_2 = nil
  L6_2 = L12_1
  L6_2 = L6_2[L4_2]
  if L6_2 ~= nil then
    L6_2 = L12_1
    L5_2 = L6_2[L4_2]
  end
  L6_2 = WIN_ENV
  if L6_2 and L5_2 == nil then
    L6_2 = NLPSession
    L6_2 = L6_2.getBackupLicense
    L6_2 = L6_2()
    L7_2 = nil
    if L6_2 ~= nil then
      L8_2 = L6_2.hash_table
      if L8_2 ~= nil then
        L8_2 = L6_2.hash_table
        L9_2 = "-k"
        L7_2 = L8_2 .. L9_2
      end
    end
    if L7_2 ~= nil then
      L8_2 = L12_1
      L8_2 = L8_2[L7_2]
      if L8_2 ~= nil then
        L8_2 = L12_1
        L5_2 = L8_2[L7_2]
      end
    end
  end
  L6_2 = L12_1
  L6_2 = L6_2.email
  if not L6_2 then
    L6_2 = nil
  end
  if L5_2 then
    L7_2 = string
    L7_2 = L7_2.match
    L8_2 = L5_2
    L9_2 = "%w%w%w%w%w%w%w%w%-%w%w%w%w%-%w%w%w%w%-%w%w%w%w%-%w%w%w%w%w%w%w%w%w%w%w%w"
    L7_2 = L7_2(L8_2, L9_2)
    if L7_2 and L6_2 ~= nil then
      L7_2 = L13_1
      L7_2 = L7_2.validEmail
      L8_2 = L6_2
      L7_2 = L7_2(L8_2)
      if L7_2 then
        goto lbl_129
      end
    end
  end
  L1_2 = true
  L7_2 = L28_1
  L8_2 = L27_1
  L9_2 = L29_1
  L7_2 = L7_2 .. L8_2 .. L9_2
  L8_2 = L8_1
  L8_2 = L8_2.exists
  L9_2 = L7_2
  L8_2 = L8_2(L9_2)
  if L8_2 then
    L8_2 = L8_1
    L8_2 = L8_2.readFile
    L9_2 = L7_2
    L8_2 = L8_2(L9_2)
    L9_2 = L13_1
    L9_2 = L9_2.lines
    L10_2 = L8_2
    L9_2 = L9_2(L10_2)
    L10_2 = #L9_2
    if L10_2 == 4 then
      L10_2 = L9_2[1]
      L11_2 = L9_2[2]
      L12_2 = L9_2[3]
      L13_2 = L9_2[4]
      L14_2 = L13_1
      L14_2 = L14_2.v3
      L14_2 = L14_2()
      if L13_2 == L14_2 then
        L14_2 = L3_1
        L14_2 = L14_2.postAsyncTaskWithContext
        L15_2 = "posting credentials"
        
        function L16_2(A0_3)
          local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3
          L1_3 = L13_1
          L1_3 = L1_3.validate
          L2_3 = L11_2
          L3_3 = L12_2
          L1_3, L2_3 = L1_3(L2_3, L3_3)
          if L1_3 == nil then
          else
            L3_3 = L17_1
            L4_3 = L3_3
            L3_3 = L3_3.decode
            L5_3 = L1_3
            L3_3 = L3_3(L4_3, L5_3)
            L4_3 = L3_3.valid
            if L4_3 == true then
              L4_3 = L3_3.instance
              L4_3 = L4_3.name
              L5_3 = L13_2
              if L4_3 == L5_3 then
                L4_3 = L3_3.meta
                L4_3 = L4_3.product_id
                if L4_3 ~= 55701 then
                  L4_3 = L3_3.meta
                  L4_3 = L4_3.product_id
                  if L4_3 ~= 64248 then
                    goto lbl_58
                  end
                end
                L4_3 = L13_1
                L4_3 = L4_3.v3
                L4_3 = L4_3()
                L5_3 = "-i"
                L4_3 = L4_3 .. L5_3
                L5_3 = L12_1
                L6_3 = L3_3.instance
                L6_3 = L6_3.id
                L5_3[L4_3] = L6_3
                L5_3 = L12_1
                L6_3 = L4_2
                L7_3 = L3_3.license_key
                L7_3 = L7_3.key
                L5_3[L6_3] = L7_3
                L5_3 = L12_1
                L6_3 = L3_3.meta
                L6_3 = L6_3.customer_email
                L5_3.email = L6_3
                L5_3 = L12_1
                L12_1 = L5_3
                L5_3 = false
                L1_2 = L5_3
                L5_3 = L4_1
                L5_3 = L5_3.message
                L6_3 = "License Successfully Restored"
                L7_3 = "Your license was successfully restored. Please restart Negative Lab Pro for the license to take effect."
                L8_3 = "info"
                L5_3(L6_3, L7_3, L8_3)
            end
            else
              ::lbl_58::
              L4_3 = L4_1
              L4_3 = L4_3.message
              L5_3 = "Unable to Restore License"
              L6_3 = "Please go to 'File > Plugin Manager' and resubmit your license"
              L7_3 = "warning"
              L4_3(L5_3, L6_3, L7_3)
              L4_3 = L8_1
              L4_3 = L4_3.move
              L5_3 = L7_2
              L6_3 = L28_1
              L7_3 = L27_1
              L8_3 = "backup-license-archive.txt"
              L6_3 = L6_3 .. L7_3 .. L8_3
              L4_3(L5_3, L6_3)
            end
          end
        end
        
        L14_2(L15_2, L16_2)
      end
    end
  end
  goto lbl_130
  ::lbl_129::
  L1_2 = false
  ::lbl_130::
  if L1_2 == true and L0_2 > L3_2 then
    L7_2 = _G
    L7_2.NLPSessionStatus = 2
  elseif L1_2 == true then
    L7_2 = _G
    L7_2.NLPSessionStatus = 1
  elseif L1_2 ~= true then
    L7_2 = _G
    L7_2.NLPSessionStatus = 3
  else
    L7_2 = _G
    L7_2.NLPSessionStatus = 0
  end
end

L34_1.setSessionStatus = L35_1
L34_1 = NLPSession

function L35_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2, L12_2, L13_2, L14_2, L15_2, L16_2, L17_2, L18_2, L19_2, L20_2, L21_2
  L0_2 = _G
  L0_2 = L0_2.activePhoto
  L1_2 = L0_2
  L0_2 = L0_2.getFormattedMetadata
  L0_2 = L0_2(L1_2)
  L1_2 = _G
  L1_2 = L1_2.activePhoto
  L2_2 = L1_2
  L1_2 = L1_2.getRawMetadata
  L1_2 = L1_2(L2_2)
  L2_2 = _G
  L2_2 = L2_2.activePhoto
  L3_2 = L2_2
  L2_2 = L2_2.getFormattedMetadata
  L4_2 = "software"
  L2_2 = L2_2(L3_2, L4_2)
  L3_2 = L1_2.fileFormat
  if L2_2 ~= nil then
  end
  if L3_2 == "TIFF" or L3_2 == "JPG" then
    L4_2 = "linearScan"
    scanSource = L4_2
  else
    if L2_2 ~= nil then
      L4_2 = string
      L4_2 = L4_2.match
      L5_2 = L2_2
      L6_2 = "VueScan"
      L4_2 = L4_2(L5_2, L6_2)
      if not L4_2 then
        L4_2 = string
        L4_2 = L4_2.match
        L5_2 = L2_2
        L6_2 = "SilverFast"
        L4_2 = L4_2(L5_2, L6_2)
      end
      if L4_2 and L3_2 == "DNG" then
        L4_2 = "vuescanRaw"
        scanSource = L4_2
    end
    elseif L3_2 == "DNG" or L3_2 == "RAW" then
      L4_2 = "cameraScan"
      scanSource = L4_2
    end
  end
  L4_2 = 1
  L5_2 = _G
  L5_2 = L5_2.activePhotos
  L5_2 = #L5_2
  L6_2 = 1
  for L7_2 = L4_2, L5_2, L6_2 do
    L8_2 = _G
    L8_2 = L8_2.activePhotos
    L8_2 = L8_2[L7_2]
    L9_2 = L8_2
    L8_2 = L8_2.getFormattedMetadata
    L10_2 = "fileName"
    L8_2 = L8_2(L9_2, L10_2)
    L9_2 = _G
    L9_2 = L9_2.activePhotos
    L9_2 = L9_2[L7_2]
    L10_2 = L9_2
    L9_2 = L9_2.getRawMetadata
    L11_2 = "croppedDimensions"
    L9_2 = L9_2(L10_2, L11_2)
    L10_2 = L9_2.width
    L11_2 = L9_2.height
    L10_2 = L10_2 * L11_2
    L10_2 = L10_2 / 1000000
    L11_2 = _G
    L11_2 = L11_2.activePhotos
    L11_2 = L11_2[L7_2]
    L12_2 = L11_2
    L11_2 = L11_2.getRawMetadata
    L13_2 = "aperture"
    L11_2 = L11_2(L12_2, L13_2)
    L12_2 = _G
    L12_2 = L12_2.activePhotos
    L12_2 = L12_2[L7_2]
    L13_2 = L12_2
    L12_2 = L12_2.getRawMetadata
    L14_2 = "shutterSpeed"
    L12_2 = L12_2(L13_2, L14_2)
    L13_2 = _G
    L13_2 = L13_2.activePhotos
    L13_2 = L13_2[L7_2]
    L14_2 = L13_2
    L13_2 = L13_2.getRawMetadata
    L15_2 = "isoSpeedRating"
    L13_2 = L13_2(L14_2, L15_2)
    L14_2 = _G
    L14_2 = L14_2.activePhotos
    L14_2 = #L14_2
    L14_2 = L7_2 / L14_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L16_2 = {}
      L15_2[L7_2] = L16_2
    end
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.previousLightroomSettings
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L16_2 = _G
      L16_2 = L16_2.activePhotos
      L16_2 = L16_2[L7_2]
      L17_2 = L16_2
      L16_2 = L16_2.getDevelopSettings
      L16_2 = L16_2(L17_2)
      L15_2.previousLightroomSettings = L16_2
      L15_2 = NLPSession
      L15_2 = L15_2.isGroupMaskAvailable
      L15_2 = L15_2()
      if L15_2 then
        L15_2 = L33_1
        L16_2 = L7_2
        L15_2(L16_2)
      end
    end
    L15_2 = L20_1
    L16_2 = L15_2
    L15_2 = L15_2.trace
    L17_2 = L15_1
    L18_2 = _G
    L18_2 = L18_2.activePhotosMeta
    L18_2 = L18_2[L7_2]
    L18_2 = L18_2.previousLightroomSettings
    L17_2, L18_2, L19_2, L20_2, L21_2 = L17_2(L18_2)
    L15_2(L16_2, L17_2, L18_2, L19_2, L20_2, L21_2)
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.roll
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotos
      L15_2 = L15_2[L7_2]
      L16_2 = L15_2
      L15_2 = L15_2.getPropertyForPlugin
      L17_2 = _PLUGIN
      L18_2 = "roll"
      L15_2, L16_2 = L15_2(L16_2, L17_2, L18_2)
      if L15_2 ~= nil then
        L17_2 = _G
        L17_2 = L17_2.activePhotosMeta
        L17_2 = L17_2[L7_2]
        L17_2.roll = L15_2
      else
      end
      L17_2 = _G
      L17_2 = L17_2.activePhotos
      L17_2 = L17_2[L7_2]
      L18_2 = L17_2
      L17_2 = L17_2.getPropertyForPlugin
      L19_2 = _PLUGIN
      L20_2 = "rollID"
      L17_2, L18_2 = L17_2(L18_2, L19_2, L20_2)
      if L17_2 then
        L19_2 = _G
        L19_2 = L19_2.activePhotosMeta
        L19_2 = L19_2[L7_2]
        L19_2.rollID = L17_2
      end
    end
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.redChannel
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L16_2 = {}
      L15_2.redChannel = L16_2
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L15_2 = L15_2.redChannel
      L15_2.settingName = "ToneCurvePV2012Red"
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L16_2 = {}
      L15_2.greenChannel = L16_2
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L15_2 = L15_2.greenChannel
      L15_2.settingName = "ToneCurvePV2012Green"
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L16_2 = {}
      L15_2.blueChannel = L16_2
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L15_2 = L15_2.blueChannel
      L15_2.settingName = "ToneCurvePV2012Blue"
      L15_2 = _G
      L15_2 = L15_2.activePhotosMeta
      L15_2 = L15_2[L7_2]
      L15_2 = L15_2.redChannel
      L15_2 = L15_2.whitePointOrigin
      if L15_2 == nil then
        L15_2 = _G
        L15_2 = L15_2.activePhotos
        L15_2 = L15_2[L7_2]
        L16_2 = L15_2
        L15_2 = L15_2.getPropertyForPlugin
        L17_2 = _PLUGIN
        L18_2 = "rgbOrigins"
        L15_2 = L15_2(L16_2, L17_2, L18_2)
        if L15_2 ~= nil then
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.redChannel
          L17_2 = L15_2.r
          L17_2 = L17_2.whitePointOrigin
          L16_2.whitePointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.redChannel
          L17_2 = L15_2.r
          L17_2 = L17_2.blackPointOrigin
          L16_2.blackPointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.redChannel
          L17_2 = L15_2.r
          L17_2 = L17_2.meanPoint
          if not L17_2 then
            L17_2 = 0.5
          end
          L16_2.meanPoint = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.greenChannel
          L17_2 = L15_2.g
          L17_2 = L17_2.whitePointOrigin
          L16_2.whitePointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.greenChannel
          L17_2 = L15_2.g
          L17_2 = L17_2.blackPointOrigin
          L16_2.blackPointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.greenChannel
          L17_2 = L15_2.g
          L17_2 = L17_2.meanPoint
          if not L17_2 then
            L17_2 = 0.5
          end
          L16_2.meanPoint = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.blueChannel
          L17_2 = L15_2.b
          L17_2 = L17_2.whitePointOrigin
          L16_2.whitePointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.blueChannel
          L17_2 = L15_2.b
          L17_2 = L17_2.blackPointOrigin
          L16_2.blackPointOrigin = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.blueChannel
          L17_2 = L15_2.b
          L17_2 = L17_2.meanPoint
          if not L17_2 then
            L17_2 = 0.5
          end
          L16_2.meanPoint = L17_2
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2.originsComputed = true
          L16_2 = true
          originsComputed = L16_2
        end
      end
    end
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.nlpVersion
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotos
      L15_2 = L15_2[L7_2]
      L16_2 = L15_2
      L15_2 = L15_2.getPropertyForPlugin
      L17_2 = _PLUGIN
      L18_2 = "nlpVersion"
      L15_2 = L15_2(L16_2, L17_2, L18_2)
      if L15_2 ~= nil then
        L16_2 = _G
        L16_2 = L16_2.activePhotosMeta
        L16_2 = L16_2[L7_2]
        L16_2.nlpVersion = L15_2
      else
        L16_2 = _G
        L16_2 = L16_2.activePhotosMeta
        L16_2 = L16_2[L7_2]
        L17_2 = _G
        L17_2 = L17_2.version
        L16_2.nlpVersion = L17_2
      end
    end
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.autoWB
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotos
      L15_2 = L15_2[L7_2]
      L16_2 = L15_2
      L15_2 = L15_2.getPropertyForPlugin
      L17_2 = _PLUGIN
      L18_2 = "autoWB"
      L15_2 = L15_2(L16_2, L17_2, L18_2)
      if L15_2 ~= nil then
        L16_2 = _G
        L16_2 = L16_2.activePhotosMeta
        L16_2 = L16_2[L7_2]
        L16_2.autoWB = L15_2
      else
      end
    end
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.megapixels = L10_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.fileFormat = L3_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.croppedDimensions = L9_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.aperture = L11_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.shutterSpeed = L12_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2.iso = L13_2
    L15_2 = _G
    L15_2 = L15_2.activePhotosMeta
    L15_2 = L15_2[L7_2]
    L15_2 = L15_2.settings
    if L15_2 == nil then
      L15_2 = _G
      L15_2 = L15_2.activePhotos
      L15_2 = L15_2[L7_2]
      L16_2 = L15_2
      L15_2 = L15_2.getPropertyForPlugin
      L17_2 = _PLUGIN
      L18_2 = "settings"
      L15_2 = L15_2(L16_2, L17_2, L18_2)
      if L15_2 ~= nil then
        L16_2 = _G
        L16_2 = L16_2.activePhotosMeta
        L16_2 = L16_2[L7_2]
        L17_2 = {}
        L16_2.settings = L17_2
        L16_2 = pairs
        L17_2 = L15_2
        L16_2, L17_2, L18_2 = L16_2(L17_2)
        for L19_2, L20_2 in L16_2, L17_2, L18_2 do
          L21_2 = _G
          L21_2 = L21_2.activePhotosMeta
          L21_2 = L21_2[L7_2]
          L21_2 = L21_2.settings
          L21_2[L19_2] = L20_2
        end
        L16_2 = L15_2.film
        if L16_2 == nil then
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.settings
          L16_2.film = "none"
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.settings
          L16_2.filmHue = 0
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.settings
          L16_2.hueStrength = 0
        end
        L16_2 = L15_2.film
        if L16_2 == "custom" then
          L16_2 = L15_2.hueStrength
          if L16_2 ~= 0 then
            L16_2 = updateHue
            L17_2 = L15_2
            L16_2, L17_2 = L16_2(L17_2)
            L18_2 = _G
            L18_2 = L18_2.activePhotosMeta
            L18_2 = L18_2[L7_2]
            L18_2 = L18_2.settings
            L18_2.wbTemp = L16_2
            L18_2 = _G
            L18_2 = L18_2.activePhotosMeta
            L18_2 = L18_2[L7_2]
            L18_2 = L18_2.settings
            L18_2.wbTint = L17_2
            L18_2 = _G
            L18_2 = L18_2.activePhotosMeta
            L18_2 = L18_2[L7_2]
            L18_2 = L18_2.settings
            L18_2.hueStrength = 0
            L18_2 = _G
            L18_2 = L18_2.activePhotosMeta
            L18_2 = L18_2[L7_2]
            L18_2 = L18_2.settings
            L18_2.filmHue = 0
          end
        end
        L16_2 = L15_2.curveResolution
        if L16_2 == nil then
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.nlpVersion
          if L16_2 ~= "2.2" then
            L16_2 = _G
            L16_2 = L16_2.activePhotosMeta
            L16_2 = L16_2[L7_2]
            L16_2 = L16_2.nlpVersion
            if L16_2 ~= "2.2.0" then
              goto lbl_481
            end
          end
          L16_2 = pairs
          L17_2 = L14_1
          L17_2 = L17_2.engine
          L17_2 = L17_2.v2_2
          L16_2, L17_2, L18_2 = L16_2(L17_2)
          for L19_2, L20_2 in L16_2, L17_2, L18_2 do
            L21_2 = _G
            L21_2 = L21_2.activePhotosMeta
            L21_2 = L21_2[L7_2]
            L21_2 = L21_2.settings
            L21_2[L19_2] = L20_2
          end
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.settings
          L16_2.enginePreset = "v2_2"
          goto lbl_500
          ::lbl_481::
          L16_2 = pairs
          L17_2 = L14_1
          L17_2 = L17_2.engine
          L17_2 = L17_2.v2_1
          L16_2, L17_2, L18_2 = L16_2(L17_2)
          for L19_2, L20_2 in L16_2, L17_2, L18_2 do
            L21_2 = _G
            L21_2 = L21_2.activePhotosMeta
            L21_2 = L21_2[L7_2]
            L21_2 = L21_2.settings
            L21_2[L19_2] = L20_2
          end
          L16_2 = _G
          L16_2 = L16_2.activePhotosMeta
          L16_2 = L16_2[L7_2]
          L16_2 = L16_2.settings
          L16_2.enginePreset = "v2_1"
        else
        end
      end
    end
    ::lbl_500::
  end
end

L34_1.setActivePhotosMetadata = L35_1

function L34_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2
  L0_2 = 1
  L1_2 = _G
  L1_2 = L1_2.activePhotos
  L1_2 = #L1_2
  L2_2 = 1
  for L3_2 = L0_2, L1_2, L2_2 do
    L4_2 = _G
    L4_2 = L4_2.activePhotosMeta
    L4_2 = L4_2[L3_2]
    L4_2 = L4_2.settings
    if L4_2 ~= nil then
      L4_2 = pairs
      L5_2 = _G
      L5_2 = L5_2.NLPDefaultSettings
      L4_2, L5_2, L6_2 = L4_2(L5_2)
      for L7_2, L8_2 in L4_2, L5_2, L6_2 do
        L9_2 = _G
        L9_2 = L9_2.activePhotosMeta
        L9_2 = L9_2[L3_2]
        L9_2 = L9_2.settings
        L9_2 = L9_2[L7_2]
        if L9_2 == nil then
          L9_2 = _G
          L9_2 = L9_2.activePhotosMeta
          L9_2 = L9_2[L3_2]
          L9_2 = L9_2.settings
          L9_2[L7_2] = L8_2
        end
      end
    else
      L4_2 = _G
      L4_2 = L4_2.activePhotosMeta
      L4_2 = L4_2[L3_2]
      L5_2 = L13_1
      L5_2 = L5_2.copyTable
      L6_2 = _G
      L6_2 = L6_2.NLPDefaultSettings
      L5_2 = L5_2(L6_2)
      L4_2.settings = L5_2
      L4_2 = scanSource
      if L4_2 ~= nil then
        L4_2 = _G
        L4_2 = L4_2.activePhotosMeta
        L4_2 = L4_2[L3_2]
        L4_2 = L4_2.settings
        L5_2 = scanSource
        L4_2.source = L5_2
      end
    end
  end
end

function L35_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2
  L0_2 = L8_1
  L0_2 = L0_2.exists
  L1_2 = L24_1
  L0_2 = L0_2(L1_2)
  if not L0_2 then
    L0_2 = L8_1
    L0_2 = L0_2.createAllDirectories
    L1_2 = L24_1
    L0_2(L1_2)
  end
  L0_2 = L17_1
  L1_2 = L0_2
  L0_2 = L0_2.encode_pretty
  L2_2 = _G
  L2_2 = L2_2.rolls
  L0_2 = L0_2(L1_2, L2_2)
  L1_2 = io
  L1_2 = L1_2.open
  L2_2 = L24_1
  L3_2 = L27_1
  L4_2 = L25_1
  L2_2 = L2_2 .. L3_2 .. L4_2
  L3_2 = "w"
  L1_2 = L1_2(L2_2, L3_2)
  L3_2 = L1_2
  L2_2 = L1_2.write
  L4_2 = L0_2
  L2_2(L3_2, L4_2)
  L3_2 = L1_2
  L2_2 = L1_2.close
  L2_2(L3_2)
end

L36_1 = NLPSession

function L37_1(A0_2)
  local L1_2, L2_2, L3_2
  L1_2 = L20_1
  L2_2 = L1_2
  L1_2 = L1_2.trace
  L3_2 = "Entering saveMetadataForAllPhotos. Waiting for write access if necessary."
  L1_2(L2_2, L3_2)
  L1_2 = L3_1
  L1_2 = L1_2.callWithContext
  L2_2 = "save metadata"
  
  function L3_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3
    L1_3 = L20_1
    L2_3 = L1_3
    L1_3 = L1_3.trace
    L3_3 = "Inside 'save metadata' function context."
    L1_3(L2_3, L3_3)
    L1_3 = L4_1
    L1_3 = L1_3.showModalProgressDialog
    L2_3 = {}
    L2_3.title = "Saving Negative Lab Pro Settings"
    L3_3 = "Preparing to save metadata for "
    L4_3 = _G
    L4_3 = L4_3.activePhotos
    L4_3 = #L4_3
    L5_3 = " photo(s)..."
    L3_3 = L3_3 .. L4_3 .. L5_3
    L2_3.caption = L3_3
    L2_3.functionContext = A0_3
    L2_3.cannotCancel = true
    L1_3 = L1_3(L2_3)
    L2_3 = L11_1
    L2_3 = L2_3.stopTracking
    L2_3()
    L2_3 = L21_1
    L3_3 = L2_3
    L2_3 = L2_3.withPrivateWriteAccessDo
    
    function L4_3(A0_4)
      local L1_4, L2_4, L3_4, L4_4, L5_4, L6_4, L7_4, L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4
      L1_4 = L20_1
      L2_4 = L1_4
      L1_4 = L1_4.trace
      L3_4 = "Acquired private write access. Starting metadata save loop."
      L1_4(L2_4, L3_4)
      L1_4 = _G
      L1_4 = L1_4.activePhotos
      L1_4 = #L1_4
      L2_4 = 1
      L3_4 = L1_4
      L4_4 = 1
      for L5_4 = L2_4, L3_4, L4_4 do
        L6_4 = L1_3
        L7_4 = L6_4
        L6_4 = L6_4.setCaption
        L8_4 = string
        L8_4 = L8_4.format
        L9_4 = "Saving photo %d of %d..."
        L10_4 = L5_4
        L11_4 = L1_4
        L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4 = L8_4(L9_4, L10_4, L11_4)
        L6_4(L7_4, L8_4, L9_4, L10_4, L11_4, L12_4, L13_4, L14_4)
        L6_4 = L9_1
        L6_4 = L6_4.yield
        L6_4()
        L6_4 = _G
        L6_4 = L6_4.activePhotosMeta
        L6_4 = L6_4[L5_4]
        if L6_4 ~= nil then
          L6_4 = _G
          L6_4 = L6_4.activePhotosMeta
          L6_4 = L6_4[L5_4]
          L6_4 = L6_4.settings
          if L6_4 ~= nil then
            L6_4 = A0_2
            if L6_4 ~= "settings" then
              L6_4 = A0_2
              if L6_4 ~= nil then
                goto lbl_93
              end
            end
            L6_4 = L20_1
            L7_4 = L6_4
            L6_4 = L6_4.trace
            L8_4 = "Saving 'settings' for photo "
            L9_4 = L5_4
            L8_4 = L8_4 .. L9_4
            L6_4(L7_4, L8_4)
            L6_4 = type
            L7_4 = _G
            L7_4 = L7_4.activePhotosMeta
            L7_4 = L7_4[L5_4]
            L7_4 = L7_4.settings
            L7_4 = L7_4.colorPicker
            L6_4 = L6_4(L7_4)
            if L6_4 == "userdata" then
              L6_4 = _G
              L6_4 = L6_4.activePhotosMeta
              L6_4 = L6_4[L5_4]
              L6_4 = L6_4.settings
              L7_4 = L13_1
              L7_4 = L7_4.colorToArray
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.settings
              L8_4 = L8_4.colorPicker
              L7_4 = L7_4(L8_4)
              L6_4.colorPicker = L7_4
            end
            L6_4 = _G
            L6_4 = L6_4.activePhotosMeta
            L6_4 = L6_4[L5_4]
            L6_4 = L6_4.settings
            L6_4 = L6_4.hueStrength
            if L6_4 ~= 0 then
              L6_4 = _G
              L6_4 = L6_4.activePhotosMeta
              L6_4 = L6_4[L5_4]
              L6_4 = L6_4.settings
              L6_4.hueStrength = 0
            end
            L6_4 = _G
            L6_4 = L6_4.activePhotos
            L6_4 = L6_4[L5_4]
            L7_4 = L6_4
            L6_4 = L6_4.setPropertyForPlugin
            L8_4 = _PLUGIN
            L9_4 = "settings"
            L10_4 = _G
            L10_4 = L10_4.activePhotosMeta
            L10_4 = L10_4[L5_4]
            L10_4 = L10_4.settings
            L6_4(L7_4, L8_4, L9_4, L10_4)
        end
        else
          ::lbl_93::
          L6_4 = A0_2
          if L6_4 ~= "settings" then
            L6_4 = A0_2
            if L6_4 ~= nil then
              goto lbl_113
            end
          end
          L6_4 = L20_1
          L7_4 = L6_4
          L6_4 = L6_4.trace
          L8_4 = "Clearing 'settings' for photo "
          L9_4 = L5_4
          L8_4 = L8_4 .. L9_4
          L6_4(L7_4, L8_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "settings"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
        end
        ::lbl_113::
        L6_4 = _G
        L6_4 = L6_4.activePhotosMeta
        L6_4 = L6_4[L5_4]
        if L6_4 ~= nil then
          L6_4 = _G
          L6_4 = L6_4.activePhotosMeta
          L6_4 = L6_4[L5_4]
          L6_4 = L6_4.redChannel
          if L6_4 ~= nil then
            L6_4 = _G
            L6_4 = L6_4.activePhotosMeta
            L6_4 = L6_4[L5_4]
            L6_4 = L6_4.redChannel
            L6_4 = L6_4.whitePointOrigin
            if L6_4 ~= nil then
              L6_4 = A0_2
              if L6_4 ~= "rgbOrigins" then
                L6_4 = A0_2
                if L6_4 ~= nil then
                  goto lbl_324
                end
              end
              L6_4 = L20_1
              L7_4 = L6_4
              L6_4 = L6_4.trace
              L8_4 = "Saving 'rgbOrigins' and related properties for photo "
              L9_4 = L5_4
              L8_4 = L8_4 .. L9_4
              L6_4(L7_4, L8_4)
              L6_4 = {}
              L7_4 = {}
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.redChannel
              L8_4 = L8_4.whitePointOrigin
              L7_4.whitePointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.redChannel
              L8_4 = L8_4.blackPointOrigin
              L7_4.blackPointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.redChannel
              L8_4 = L8_4.meanPoint
              L7_4.meanPoint = L8_4
              L6_4.r = L7_4
              L7_4 = {}
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.greenChannel
              L8_4 = L8_4.whitePointOrigin
              L7_4.whitePointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.greenChannel
              L8_4 = L8_4.blackPointOrigin
              L7_4.blackPointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.greenChannel
              L8_4 = L8_4.meanPoint
              L7_4.meanPoint = L8_4
              L6_4.g = L7_4
              L7_4 = {}
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.blueChannel
              L8_4 = L8_4.whitePointOrigin
              L7_4.whitePointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.blueChannel
              L8_4 = L8_4.blackPointOrigin
              L7_4.blackPointOrigin = L8_4
              L8_4 = _G
              L8_4 = L8_4.activePhotosMeta
              L8_4 = L8_4[L5_4]
              L8_4 = L8_4.blueChannel
              L8_4 = L8_4.meanPoint
              L7_4.meanPoint = L8_4
              L6_4.b = L7_4
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "rgbOrigins"
              L11_4 = L6_4
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpConverted"
              L11_4 = "converted"
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpVersion"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.nlpVersion
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpSource"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.settings
              L11_4 = L11_4.source
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpColorModel"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.settings
              L11_4 = L11_4.colorModel
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpPreSaturation"
              L11_4 = tostring
              L12_4 = _G
              L12_4 = L12_4.activePhotosMeta
              L12_4 = L12_4[L5_4]
              L12_4 = L12_4.settings
              L12_4 = L12_4.saturation
              L11_4 = L11_4(L12_4)
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpTones"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.settings
              L11_4 = L11_4.toneProfile
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpColor"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.settings
              L11_4 = L11_4.film
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "nlpLUT"
              L11_4 = _G
              L11_4 = L11_4.activePhotosMeta
              L11_4 = L11_4[L5_4]
              L11_4 = L11_4.settings
              L11_4 = L11_4.enhancedProfile
              if not L11_4 then
                L11_4 = nil
              end
              L7_4(L8_4, L9_4, L10_4, L11_4)
          end
        end
        else
          ::lbl_324::
          L6_4 = A0_2
          if L6_4 ~= "rgbOrigins" then
            L6_4 = A0_2
            if L6_4 ~= nil then
              goto lbl_408
            end
          end
          L6_4 = L20_1
          L7_4 = L6_4
          L6_4 = L6_4.trace
          L8_4 = "Clearing 'rgbOrigins' and related properties for photo "
          L9_4 = L5_4
          L8_4 = L8_4 .. L9_4
          L6_4(L7_4, L8_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "rgbOrigins"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpConverted"
          L10_4 = "non_converted"
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpVersion"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpSource"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpColorModel"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpPreSaturation"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpTones"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpColor"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "nlpLUT"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
        end
        ::lbl_408::
        L6_4 = _G
        L6_4 = L6_4.activePhotosMeta
        L6_4 = L6_4[L5_4]
        if L6_4 ~= nil then
          L6_4 = _G
          L6_4 = L6_4.activePhotosMeta
          L6_4 = L6_4[L5_4]
          L6_4 = L6_4.settings
          if L6_4 ~= nil then
            L6_4 = _G
            L6_4 = L6_4.activePhotosMeta
            L6_4 = L6_4[L5_4]
            L6_4 = L6_4.settings
            L6_4 = L6_4.rollAnalysis
            L7_4 = L20_1
            L8_4 = L7_4
            L7_4 = L7_4.trace
            L9_4 = "Checking roll analysis for photo "
            L10_4 = L5_4
            L11_4 = ": "
            L12_4 = tostring
            L13_4 = L6_4
            L12_4 = L12_4(L13_4)
            L9_4 = L9_4 .. L10_4 .. L11_4 .. L12_4
            L7_4(L8_4, L9_4)
            if L6_4 == "individual" or L6_4 == nil then
              L7_4 = L20_1
              L8_4 = L7_4
              L7_4 = L7_4.trace
              L9_4 = "Clearing roll metadata for photo "
              L10_4 = L5_4
              L9_4 = L9_4 .. L10_4
              L7_4(L8_4, L9_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "rollName"
              L11_4 = nil
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "rollID"
              L11_4 = nil
              L7_4(L8_4, L9_4, L10_4, L11_4)
              L7_4 = _G
              L7_4 = L7_4.activePhotos
              L7_4 = L7_4[L5_4]
              L8_4 = L7_4
              L7_4 = L7_4.setPropertyForPlugin
              L9_4 = _PLUGIN
              L10_4 = "roll"
              L11_4 = nil
              L7_4(L8_4, L9_4, L10_4, L11_4)
            else
              L7_4 = "rollID_"
              L8_4 = L6_4
              L7_4 = L7_4 .. L8_4
              L8_4 = _G
              L8_4 = L8_4.rolls
              if L8_4 then
                L8_4 = _G
                L8_4 = L8_4.rolls
                L8_4 = L8_4[L7_4]
                if L8_4 then
                  L8_4 = _G
                  L8_4 = L8_4.rolls
                  L8_4 = L8_4[L7_4]
                  L9_4 = L20_1
                  L10_4 = L9_4
                  L9_4 = L9_4.trace
                  L11_4 = "Saving roll metadata from _G.rolls for photo "
                  L12_4 = L5_4
                  L13_4 = ": "
                  L14_4 = L8_4.rollName
                  L11_4 = L11_4 .. L12_4 .. L13_4 .. L14_4
                  L9_4(L10_4, L11_4)
                  L9_4 = _G
                  L9_4 = L9_4.activePhotos
                  L9_4 = L9_4[L5_4]
                  L10_4 = L9_4
                  L9_4 = L9_4.setPropertyForPlugin
                  L11_4 = _PLUGIN
                  L12_4 = "rollName"
                  L13_4 = L8_4.rollName
                  L9_4(L10_4, L11_4, L12_4, L13_4)
                  L9_4 = _G
                  L9_4 = L9_4.activePhotos
                  L9_4 = L9_4[L5_4]
                  L10_4 = L9_4
                  L9_4 = L9_4.setPropertyForPlugin
                  L11_4 = _PLUGIN
                  L12_4 = "rollID"
                  L13_4 = L8_4.ID
                  L9_4(L10_4, L11_4, L12_4, L13_4)
                  L9_4 = _G
                  L9_4 = L9_4.activePhotos
                  L9_4 = L9_4[L5_4]
                  L10_4 = L9_4
                  L9_4 = L9_4.setPropertyForPlugin
                  L11_4 = _PLUGIN
                  L12_4 = "roll"
                  L13_4 = L8_4
                  L9_4(L10_4, L11_4, L12_4, L13_4)
              end
              else
                L8_4 = _G
                L8_4 = L8_4.activePhotosMeta
                L8_4 = L8_4[L5_4]
                L8_4 = L8_4.rollName
                if L8_4 ~= nil then
                  L8_4 = _G
                  L8_4 = L8_4.activePhotosMeta
                  L8_4 = L8_4[L5_4]
                  L8_4 = L8_4.rollID
                  if L8_4 ~= nil then
                    L8_4 = _G
                    L8_4 = L8_4.activePhotosMeta
                    L8_4 = L8_4[L5_4]
                    L8_4 = L8_4.roll
                    if L8_4 ~= nil then
                      L8_4 = L20_1
                      L9_4 = L8_4
                      L8_4 = L8_4.trace
                      L10_4 = "Saving temporary roll metadata for photo "
                      L11_4 = L5_4
                      L12_4 = ": "
                      L13_4 = _G
                      L13_4 = L13_4.activePhotosMeta
                      L13_4 = L13_4[L5_4]
                      L13_4 = L13_4.rollName
                      L10_4 = L10_4 .. L11_4 .. L12_4 .. L13_4
                      L8_4(L9_4, L10_4)
                      L8_4 = _G
                      L8_4 = L8_4.activePhotos
                      L8_4 = L8_4[L5_4]
                      L9_4 = L8_4
                      L8_4 = L8_4.setPropertyForPlugin
                      L10_4 = _PLUGIN
                      L11_4 = "rollName"
                      L12_4 = _G
                      L12_4 = L12_4.activePhotosMeta
                      L12_4 = L12_4[L5_4]
                      L12_4 = L12_4.rollName
                      L8_4(L9_4, L10_4, L11_4, L12_4)
                      L8_4 = _G
                      L8_4 = L8_4.activePhotos
                      L8_4 = L8_4[L5_4]
                      L9_4 = L8_4
                      L8_4 = L8_4.setPropertyForPlugin
                      L10_4 = _PLUGIN
                      L11_4 = "rollID"
                      L12_4 = _G
                      L12_4 = L12_4.activePhotosMeta
                      L12_4 = L12_4[L5_4]
                      L12_4 = L12_4.rollID
                      L8_4(L9_4, L10_4, L11_4, L12_4)
                      L8_4 = _G
                      L8_4 = L8_4.activePhotos
                      L8_4 = L8_4[L5_4]
                      L9_4 = L8_4
                      L8_4 = L8_4.setPropertyForPlugin
                      L10_4 = _PLUGIN
                      L11_4 = "roll"
                      L12_4 = _G
                      L12_4 = L12_4.activePhotosMeta
                      L12_4 = L12_4[L5_4]
                      L12_4 = L12_4.roll
                      L8_4(L9_4, L10_4, L11_4, L12_4)
                  end
                end
                else
                  L8_4 = L20_1
                  L9_4 = L8_4
                  L8_4 = L8_4.warn
                  L10_4 = "Could not find roll data for rollAnalysis ID: "
                  L11_4 = L6_4
                  L12_4 = " for photo "
                  L13_4 = L5_4
                  L10_4 = L10_4 .. L11_4 .. L12_4 .. L13_4
                  L8_4(L9_4, L10_4)
                  L8_4 = _G
                  L8_4 = L8_4.activePhotos
                  L8_4 = L8_4[L5_4]
                  L9_4 = L8_4
                  L8_4 = L8_4.setPropertyForPlugin
                  L10_4 = _PLUGIN
                  L11_4 = "rollName"
                  L12_4 = nil
                  L8_4(L9_4, L10_4, L11_4, L12_4)
                  L8_4 = _G
                  L8_4 = L8_4.activePhotos
                  L8_4 = L8_4[L5_4]
                  L9_4 = L8_4
                  L8_4 = L8_4.setPropertyForPlugin
                  L10_4 = _PLUGIN
                  L11_4 = "rollID"
                  L12_4 = nil
                  L8_4(L9_4, L10_4, L11_4, L12_4)
                  L8_4 = _G
                  L8_4 = L8_4.activePhotos
                  L8_4 = L8_4[L5_4]
                  L9_4 = L8_4
                  L8_4 = L8_4.setPropertyForPlugin
                  L10_4 = _PLUGIN
                  L11_4 = "roll"
                  L12_4 = nil
                  L8_4(L9_4, L10_4, L11_4, L12_4)
                end
              end
            end
        end
        else
          L6_4 = L20_1
          L7_4 = L6_4
          L6_4 = L6_4.trace
          L8_4 = "Skipping roll metadata save for photo "
          L9_4 = L5_4
          L10_4 = " (no settings found)."
          L8_4 = L8_4 .. L9_4 .. L10_4
          L6_4(L7_4, L8_4)
        end
        L6_4 = _G
        L6_4 = L6_4.activePhotosMeta
        L6_4 = L6_4[L5_4]
        if L6_4 ~= nil then
          L6_4 = _G
          L6_4 = L6_4.activePhotosMeta
          L6_4 = L6_4[L5_4]
          L6_4 = L6_4.autoWB
          if L6_4 ~= nil then
            L6_4 = A0_2
            if L6_4 ~= "autoWB" then
              L6_4 = A0_2
              if L6_4 ~= nil then
                goto lbl_655
              end
            end
            L6_4 = L20_1
            L7_4 = L6_4
            L6_4 = L6_4.trace
            L8_4 = "Saving 'autoWB' for photo "
            L9_4 = L5_4
            L8_4 = L8_4 .. L9_4
            L6_4(L7_4, L8_4)
            L6_4 = _G
            L6_4 = L6_4.activePhotos
            L6_4 = L6_4[L5_4]
            L7_4 = L6_4
            L6_4 = L6_4.setPropertyForPlugin
            L8_4 = _PLUGIN
            L9_4 = "autoWB"
            L10_4 = _G
            L10_4 = L10_4.activePhotosMeta
            L10_4 = L10_4[L5_4]
            L10_4 = L10_4.autoWB
            L6_4(L7_4, L8_4, L9_4, L10_4)
        end
        else
          ::lbl_655::
          L6_4 = A0_2
          if L6_4 ~= "autoWB" then
            L6_4 = A0_2
            if L6_4 ~= nil then
              goto lbl_675
            end
          end
          L6_4 = L20_1
          L7_4 = L6_4
          L6_4 = L6_4.trace
          L8_4 = "Clearing 'autoWB' for photo "
          L9_4 = L5_4
          L8_4 = L8_4 .. L9_4
          L6_4(L7_4, L8_4)
          L6_4 = _G
          L6_4 = L6_4.activePhotos
          L6_4 = L6_4[L5_4]
          L7_4 = L6_4
          L6_4 = L6_4.setPropertyForPlugin
          L8_4 = _PLUGIN
          L9_4 = "autoWB"
          L10_4 = nil
          L6_4(L7_4, L8_4, L9_4, L10_4)
        end
        ::lbl_675::
        L6_4 = L1_3
        L7_4 = L6_4
        L6_4 = L6_4.setPortionComplete
        L8_4 = L5_4
        L9_4 = L1_4
        L6_4(L7_4, L8_4, L9_4)
      end
      L2_4 = L20_1
      L3_4 = L2_4
      L2_4 = L2_4.trace
      L4_4 = "Finished metadata save loop."
      L2_4(L3_4, L4_4)
      L2_4 = true
      return L2_4
    end
    
    L5_3 = {}
    L5_3.timeout = 60
    L5_3.progressScope = L1_3
    L2_3, L3_3 = L2_3(L3_3, L4_3, L5_3)
    if L2_3 then
      L4_3 = L20_1
      L5_3 = L4_3
      L4_3 = L4_3.trace
      L6_3 = "Successfully completed catalog:withPrivateWriteAccessDo."
      L4_3(L5_3, L6_3)
      L5_3 = L1_3
      L4_3 = L1_3.setCaption
      L6_3 = "Save complete!"
      L4_3(L5_3, L6_3)
    else
      L4_3 = L20_1
      L5_3 = L4_3
      L4_3 = L4_3.error
      L6_3 = "Failed to save metadata. catalog:withPrivateWriteAccessDo returned false. Message: "
      L7_3 = L3_3 or L7_3
      if not L3_3 then
        L7_3 = "Unknown error"
      end
      L6_3 = L6_3 .. L7_3
      L4_3(L5_3, L6_3)
      L4_3 = L4_1
      L4_3 = L4_3.showError
      L5_3 = "Failed to save Negative Lab Pro metadata to the catalog. Please check the log file for details. Error: "
      L6_3 = L3_3 or L6_3
      if not L3_3 then
        L6_3 = "Unknown error"
      end
      L5_3 = L5_3 .. L6_3
      L4_3(L5_3)
      L5_3 = L1_3
      L4_3 = L1_3.setCaption
      L6_3 = "Error during save!"
      L4_3(L5_3, L6_3)
    end
    L5_3 = L1_3
    L4_3 = L1_3.done
    L4_3(L5_3)
    L4_3 = L20_1
    L5_3 = L4_3
    L4_3 = L4_3.trace
    L6_3 = "Modal progress dialog marked as done."
    L4_3(L5_3, L6_3)
  end
  
  L1_2(L2_2, L3_2)
  L1_2 = L20_1
  L2_2 = L1_2
  L1_2 = L1_2.trace
  L3_2 = "Exiting saveMetadataForAllPhotos."
  L1_2(L2_2, L3_2)
end

L36_1.saveMetadataForAllPhotos = L37_1
L36_1 = NLPSession

function L37_1(A0_2)
  local L1_2
  L1_2 = _G
  L1_2 = L1_2.activePhotosMeta
  L1_2 = L1_2[A0_2]
  if L1_2 ~= nil then
    L1_2 = _G
    L1_2 = L1_2.activePhotosMeta
    L1_2 = L1_2[A0_2]
    L1_2 = L1_2.redChannel
    L1_2 = L1_2.whitePointOrigin
    if L1_2 ~= nil then
      L1_2 = _G
      L1_2.isConverted = true
      L1_2 = true
      return L1_2
  end
  else
    L1_2 = _G
    L1_2.isConverted = false
    L1_2 = false
    return L1_2
  end
end

L36_1.isConverted = L37_1
L36_1 = NLPSession

function L37_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2, L8_2, L9_2, L10_2, L11_2
  L0_2 = _G
  L0_2 = L0_2.isConverted
  L1_2 = false
  L2_2 = _G
  L2_2 = L2_2.activePhotos
  L2_2 = #L2_2
  if L2_2 == 1 or L0_2 == true then
    return
  end
  L2_2 = _G
  L2_2 = L2_2.activePhotos
  L2_2 = #L2_2
  L3_2 = 1
  L4_2 = -1
  for L5_2 = L2_2, L3_2, L4_2 do
    L6_2 = L20_1
    L7_2 = L6_2
    L6_2 = L6_2.trace
    L8_2 = "Checking "
    L9_2 = L5_2
    L10_2 = " of "
    L11_2 = _G
    L11_2 = L11_2.activePhotos
    L11_2 = #L11_2
    L8_2 = L8_2 .. L9_2 .. L10_2 .. L11_2
    L6_2(L7_2, L8_2)
    L6_2 = NLPSession
    L6_2 = L6_2.isConverted
    L7_2 = L5_2
    L6_2 = L6_2(L7_2)
    if L6_2 ~= L0_2 then
      L6_2 = L20_1
      L7_2 = L6_2
      L6_2 = L6_2.trace
      L8_2 = "Mismatched conversion found for photo "
      L9_2 = L5_2
      L10_2 = " of "
      L11_2 = _G
      L11_2 = L11_2.activePhotos
      L11_2 = #L11_2
      L8_2 = L8_2 .. L9_2 .. L10_2 .. L11_2
      L6_2(L7_2, L8_2)
      L6_2 = table
      L6_2 = L6_2.remove
      L7_2 = _G
      L7_2 = L7_2.activePhotos
      L8_2 = L5_2
      L6_2(L7_2, L8_2)
      L6_2 = table
      L6_2 = L6_2.remove
      L7_2 = _G
      L7_2 = L7_2.activePhotosMeta
      L8_2 = L5_2
      L6_2(L7_2, L8_2)
      L1_2 = true
    end
  end
  if L1_2 == true then
    L2_2 = L21_1
    L3_2 = L2_2
    L2_2 = L2_2.setSelectedPhotos
    L4_2 = _G
    L4_2 = L4_2.activePhoto
    L5_2 = _G
    L5_2 = L5_2.activePhotos
    L2_2(L3_2, L4_2, L5_2)
  end
  L2_2 = NLPSession
  L2_2 = L2_2.setActivePhotos
  L2_2()
  L2_2 = _G
  L2_2.isConverted = L0_2
end

L36_1.checkBatchConversionStatus = L37_1
L36_1 = NLPSession

function L37_1()
  local L0_2, L1_2, L2_2
  L0_2 = L20_1
  L1_2 = L0_2
  L0_2 = L0_2.trace
  L2_2 = "Loading photos"
  L0_2(L1_2, L2_2)
  L0_2 = NLPSession
  L0_2 = L0_2.setActivePhotos
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.setActivePhotosMetadata
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.isConverted
  L1_2 = _G
  L1_2 = L1_2.activePhotoIndex
  L0_2(L1_2)
  L0_2 = NLPSession
  L0_2 = L0_2.checkBatchConversionStatus
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.setHasSettings
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.setIsMonochrome
  L0_2()
  L0_2 = loadRollFile
  L0_2()
  L0_2 = loadPresetFiles
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.buildDefaultSettings
  L0_2()
  L0_2 = _G
  L0_2 = L0_2.buildtype
  if L0_2 then
    L0_2 = string
    L0_2 = L0_2.match
    L1_2 = _G
    L1_2 = L1_2.buildtype
    L2_2 = "multi"
    L0_2 = L0_2(L1_2, L2_2)
    if L0_2 then
      L0_2 = L32_1
      L0_2()
  end
  else
    L0_2 = NLPSession
    L0_2 = L0_2.setSessionStatus
    L0_2()
  end
  L0_2 = L34_1
  L0_2()
end

L36_1.startSession = L37_1
L36_1 = NLPSession

function L37_1()
  local L0_2, L1_2, L2_2
  L0_2 = L20_1
  L1_2 = L0_2
  L0_2 = L0_2.trace
  L2_2 = "End Session Started"
  L0_2(L1_2, L2_2)
  L0_2 = _G
  L0_2 = L0_2.rollAdded
  if not L0_2 then
    L0_2 = _G
    L0_2 = L0_2.rollRemoved
    if not L0_2 then
      goto lbl_15
    end
  end
  L0_2 = L35_1
  L0_2()
  ::lbl_15::
  L0_2 = L30_1
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.saveMetadataForAllPhotos
  L0_2()
  L0_2 = NLPSession
  L0_2 = L0_2.clearSession
  L0_2()
  L0_2 = L20_1
  L1_2 = L0_2
  L0_2 = L0_2.trace
  L2_2 = "End Session Finished"
  L0_2(L1_2, L2_2)
end

L36_1.endSession = L37_1
L36_1 = NLPSession
return L36_1
