local L1_1, L2_1, L3_1

L1_1 = require
L2_1 = "NLPDialogsV3"
L1_1 = L1_1(L2_1)
L2_1 = require
L3_1 = "NLPSessionV3"
L2_1 = L2_1(L3_1)
L3_1 = _G
L3_1.tempMultiplier = 0.75
L3_1 = L2_1.clearSession
L3_1()
L3_1 = L1_1.mainDialog
L3_1()
