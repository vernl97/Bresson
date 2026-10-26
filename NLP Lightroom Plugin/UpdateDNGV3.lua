local L0_1, L1_1, L2_1, L3_1, L4_1, L5_1, L6_1, L7_1, L8_1, L9_1, L10_1, L11_1, L12_1, L13_1, L14_1, L15_1, L16_1, L17_1, L18_1
L0_1 = "Hi friends. Nate here. This is just a quick note to ask that you please respect the time and effort I put into creating this software. Negative Lab Pro is not made or sold by a giant corporation. It is just me. It is the way that I personally provide for my family. Decompiling or modifying this software not only violates copyright law, but it directly hurts my ablity to keep making and improving this software for the film community. I understand that not everyone is in a position to purchase a Negative Lab Pro license. I do offer a 50% discount for students. And if your financial situation prohibits you from using Negative Lab Pro, please email me at nate@natephotographic.com. But please, leave this software as is so that I can continue making software for the film community. Thanks! -Nate"
L1_1 = import
L2_1 = "LrApplication"
L1_1 = L1_1(L2_1)
L2_1 = import
L3_1 = "LrTasks"
L2_1 = L2_1(L3_1)
L3_1 = require
L4_1 = "ExifV3"
L3_1 = L3_1(L4_1)
L4_1 = import
L5_1 = "LrDialogs"
L4_1 = L4_1(L5_1)
L5_1 = import
L6_1 = "LrLogger"
L5_1 = L5_1(L6_1)
L6_1 = import
L7_1 = "LrFunctionContext"
L6_1 = L6_1(L7_1)
L7_1 = import
L8_1 = "LrProgressScope"
L7_1 = L7_1(L8_1)
L8_1 = import
L9_1 = "LrApplicationView"
L8_1 = L8_1(L9_1)
L9_1 = L8_1.getCurrentModuleName
L9_1 = L9_1()
L10_1 = L5_1
L11_1 = "NegativeExport"
L10_1 = L10_1(L11_1)
L11_1 = _G
L11_1 = L11_1.dev
if L11_1 == true then
  L12_1 = L10_1
  L11_1 = L10_1.enable
  L13_1 = "logfile"
  L11_1(L12_1, L13_1)
else
  L12_1 = L10_1
  L11_1 = L10_1.disable
  L11_1(L12_1)
end
L11_1 = ""
if L9_1 == "develop" then
  L11_1 = "'Photo > Read Metadata From File'"
else
  L11_1 = "'Metadata > Read Metadata From File'"
end
L12_1 = "Negative Lab v2.3"
L13_1 = "\""
L14_1 = L1_1.activeCatalog
L14_1 = L14_1()
L15_1 = L14_1.targetPhotos

function L16_1(A0_2, A1_2)
  local L2_2, L3_2, L4_2, L5_2, L6_2
  L2_2 = L6_1
  L2_2 = L2_2.callWithContext
  L3_2 = "update DNG"
  
  function L4_2(A0_3)
    local L1_3, L2_3, L3_3, L4_3, L5_3, L6_3, L7_3, L8_3, L9_3, L10_3, L11_3, L12_3
    L1_3 = L4_1
    L1_3 = L1_3.showModalProgressDialog
    L2_3 = {}
    L2_3.title = "Updating Metadata"
    L2_3.caption = "This may take a minute..."
    L2_3.cannotCancel = true
    L2_3.functionContext = A0_3
    L1_3 = L1_3(L2_3)
    L3_3 = L1_3
    L2_3 = L1_3.setPortionComplete
    L4_3 = 0.33
    L5_3 = 1
    L2_3(L3_3, L4_3, L5_3)
    L2_3 = MAC_ENV
    if L2_3 then
      L2_3 = " -m -overwrite_original -E "
      if L2_3 then
        goto lbl_20
      end
    end
    L2_3 = " -charset Latin -m -overwrite_original -E "
    ::lbl_20::
    L3_3 = " -UniqueCameraModel="
    L4_3 = L13_1
    L5_3 = A0_2
    L6_3 = L13_1
    L7_3 = " -xmp:CameraProfile="
    L8_3 = L13_1
    L9_3 = L12_1
    L10_3 = L13_1
    L3_3 = L3_3 .. L4_3 .. L5_3 .. L6_3 .. L7_3 .. L8_3 .. L9_3 .. L10_3
    L4_3 = A1_2
    if L4_3 == true then
      L4_3 = L3_3
      L5_3 = " -software="
      L6_3 = "VueScan"
      L3_3 = L4_3 .. L5_3 .. L6_3
    end
    L4_3 = {}
    L5_3 = 1
    L6_3 = L15_1
    L6_3 = #L6_3
    L7_3 = 1
    for L8_3 = L5_3, L6_3, L7_3 do
      L9_3 = L15_1
      L9_3 = L9_3[L8_3]
      L10_3 = L9_3
      L9_3 = L9_3.getRawMetadata
      L11_3 = "path"
      L9_3 = L9_3(L10_3, L11_3)
      L10_3 = table
      L10_3 = L10_3.insert
      L11_3 = L4_3
      L12_3 = L9_3
      L10_3(L11_3, L12_3)
    end
    L5_3 = L3_1
    L5_3 = L5_3.execute
    L6_3 = L2_3
    L7_3 = L3_3
    L8_3 = L4_3
    L5_3(L6_3, L7_3, L8_3)
    L6_3 = L1_3
    L5_3 = L1_3.setPortionComplete
    L7_3 = 0.8
    L8_3 = 1
    L5_3(L6_3, L7_3, L8_3)
  end
  
  L2_2(L3_2, L4_2)
  L2_2 = L2_1
  L2_2 = L2_2.sleep
  L3_2 = 0.3
  L2_2(L3_2)
  L2_2 = L4_1
  L2_2 = L2_2.message
  L3_2 = "Almost there..."
  L4_2 = "To sync this update, click "
  L5_2 = L11_1
  L6_2 = ". It may take a few moments for the changes to sync. You should then be able to select the 'Negative Lab' camera profile"
  L4_2 = L4_2 .. L5_2 .. L6_2
  L2_2(L3_2, L4_2)
end

L17_1 = L2_1.startAsyncTask

function L18_1()
  local L0_2, L1_2, L2_2, L3_2, L4_2, L5_2, L6_2, L7_2
  L0_2 = L15_1
  L0_2 = L0_2[1]
  L1_2 = L0_2
  L0_2 = L0_2.getRawMetadata
  L0_2 = L0_2(L1_2)
  L1_2 = L15_1
  L1_2 = L1_2[1]
  L2_2 = L1_2
  L1_2 = L1_2.getFormattedMetadata
  L3_2 = "software"
  L1_2 = L1_2(L2_2, L3_2)
  L2_2 = L0_2.fileFormat
  if L2_2 == "DNG" then
    if L1_2 ~= nil then
      L3_2 = string
      L3_2 = L3_2.match
      L4_2 = L1_2
      L5_2 = "VueScan"
      L3_2 = L3_2(L4_2, L5_2)
      if L3_2 then
        L3_2 = L4_1
        L3_2 = L3_2.confirm
        L4_2 = "OK! Ready to update Vuescan DNGs?"
        L5_2 = "Vuescan DNG(s) detected. Press OK to continue..."
        L3_2 = L3_2(L4_2, L5_2)
        if L3_2 == "ok" then
          L4_2 = L16_1
          L5_2 = "Vuescan RAW"
          L4_2(L5_2)
        end
    end
    else
      if L1_2 ~= nil then
        L3_2 = string
        L3_2 = L3_2.match
        L4_2 = L1_2
        L5_2 = "SilverFast"
        L3_2 = L3_2(L4_2, L5_2)
        if L3_2 then
          L3_2 = L4_1
          L3_2 = L3_2.confirm
          L4_2 = "OK! Ready to update SilverFast DNGs?"
          L5_2 = "Silverfast DNG(s) detected. Press OK to continue..."
          L3_2 = L3_2(L4_2, L5_2)
          if L3_2 == "ok" then
            L4_2 = L16_1
            L5_2 = "Silverfast RAW"
            L4_2(L5_2)
          end
      end
      else
        L3_2 = L4_1
        L3_2 = L3_2.confirm
        L4_2 = "WARNING: This utility is for Vuescan or Silverfast DNGs only"
        L5_2 = "We can't confirm that this DNG was made in Vuescan or Silverfast. Running this utility on DNGs that were not made in Vuescan or Silverfast could have unexpected results on your files."
        L6_2 = "OK, exit this utility"
        L7_2 = "Run utility anyway"
        L3_2 = L3_2(L4_2, L5_2, L6_2, L7_2)
        if L3_2 == "cancel" then
          L4_2 = L16_1
          L5_2 = "Vuescan RAW"
          L6_2 = true
          L4_2(L5_2, L6_2)
        end
      end
    end
  else
    L3_2 = L4_1
    L3_2 = L3_2.message
    L4_2 = "Wrong File Type"
    L5_2 = "This utility only works on RAW DNG files. Please select RAW DNG files and try again."
    L6_2 = "critical"
    L3_2(L4_2, L5_2, L6_2)
  end
end

L17_1(L18_1)
