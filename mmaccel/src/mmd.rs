#![allow(dead_code)]

use windows::Win32::{Graphics::Direct3D::D3DMATRIX, Graphics::Direct3D9::D3DMATERIAL9};

extern "system" {
    pub fn ExpGetFrameTime() -> f32;
    pub fn ExpGetPmdNum() -> i32;
    pub fn ExpGetPmdFilename(model: i32) -> *const u8;
    pub fn ExpGetPmdOrder(model: i32) -> i32;
    pub fn ExpGetPmdMatNum(model: i32) -> i32;
    pub fn ExpGetPmdMaterial(model: i32, material: i32) -> D3DMATERIAL9;
    pub fn ExpGetPmdBoneNum(model: i32) -> i32;
    pub fn ExpGetPmdBoneName(model: i32, bone: i32) -> *const u8;
    pub fn ExpGetPmdBoneWorldMat(model: i32, bone: i32) -> D3DMATRIX;
    pub fn ExpGetPmdMorphNum(model: i32) -> i32;
    pub fn ExpGetPmdMorphName(model: i32, morph: i32) -> *const u8;
    pub fn ExpGetPmdMorphValue(model: i32, morph: i32) -> f32;
    pub fn ExpGetPmdDisp(model: i32) -> bool;
    pub fn ExpGetPmdID(model: i32) -> i32;
    pub fn ExpGetAcsNum() -> i32;
    pub fn ExpGetPreAcsNum() -> i32;
    pub fn ExpGetAcsFilename(acs: i32) -> *const u8;
    pub fn ExpGetAcsOrder(acs: i32) -> i32;
    pub fn ExpGetAcsWorldMat(acs: i32) -> D3DMATRIX;
    pub fn ExpGetAcsX(acs: i32) -> f32;
    pub fn ExpGetAcsY(acs: i32) -> f32;
    pub fn ExpGetAcsZ(acs: i32) -> f32;
    pub fn ExpGetAcsRx(acs: i32) -> f32;
    pub fn ExpGetAcsRy(acs: i32) -> f32;
    pub fn ExpGetAcsRz(acs: i32) -> f32;
    pub fn ExpGetAcsSi(acs: i32) -> f32;
    pub fn ExpGetAcsTr(acs: i32) -> f32;
    pub fn ExpGetAcsDisp(acs: i32) -> bool;
    pub fn ExpGetAcsID(acs: i32) -> i32;
    pub fn ExpGetAcsMatNum(acs: i32) -> i32;
    pub fn ExpGetAcsMaterial(asc: i32, material: i32) -> D3DMATERIAL9;
    pub fn ExpGetCurrentObject() -> i32;
    pub fn ExpGetCurrentMaterial() -> i32;
    pub fn ExpGetCurrentTechnic() -> i32;
    pub fn ExpSetRenderRepeatCount(n: i32);
    pub fn ExpGetRenderRepeatCount() -> i32;
    pub fn ExpGetEnglishMode() -> bool;
}
