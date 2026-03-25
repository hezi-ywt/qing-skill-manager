use super::{now_timestamp, read_package_state, write_package_state};
use crate::types::{
    CreateVariantRequest, CreateVariantResponse, DeleteVariantRequest, SkillVariant,
    UpdateVariantRequest,
};
use crate::utils::path::sanitize_dir_name;

#[tauri::command]
pub fn create_skill_variant(
    request: CreateVariantRequest,
) -> Result<CreateVariantResponse, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = SkillVariant {
        id: format!("{}-{}", request.skill_id, sanitize_dir_name(&request.name)),
        name: request.name,
        current_version: request.version_id,
        created_at: now_timestamp(),
        description: request.description,
    };
    state.variants.push(variant.clone());
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(CreateVariantResponse { variant })
}

#[tauri::command]
pub fn update_skill_variant(request: UpdateVariantRequest) -> Result<SkillVariant, String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let variant = state
        .variants
        .iter_mut()
        .find(|variant| variant.id == request.variant_id)
        .ok_or_else(|| "Variant not found".to_string())?;

    if let Some(new_name) = request.new_name {
        variant.name = new_name;
    }
    if let Some(new_version_id) = request.new_version_id {
        variant.current_version = new_version_id;
    }
    if request.new_description.is_some() {
        variant.description = request.new_description;
    }

    let updated = variant.clone();
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_skill_variant(request: DeleteVariantRequest) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Unable to determine the home directory")?;
    let mut state = read_package_state(&home, &request.skill_id);
    let before = state.variants.len();
    state.variants.retain(|variant| variant.id != request.variant_id);
    if before == state.variants.len() {
        return Err("Variant not found".to_string());
    }
    write_package_state(&home, &request.skill_id, &state)?;
    Ok(())
}
