//! Deterministic apprenticeship, school, archive, and custodianship services.

use crate::data::GameData;
use crate::state::sim::{
    Apprenticeship, CrewMember, InstitutionRecord, InstitutionRecordKind, ProcedureArchive,
    SimState, SubsystemSchool,
};

pub fn discipline_for_post(post_id: &str) -> &'static str {
    match post_id {
        "engineer" => "engineering_bay",
        "medic" => "medical_bay",
        "agronomist" => "agriculture",
        "security_chief" | "commander" => "security",
        "scientist" => "life_support_habitat",
        "navigator" => "education_culture",
        _ => "education_culture",
    }
}

pub fn designate_apprentice(
    sim: &mut SimState,
    data: &GameData,
    post_id: &str,
) -> Result<String, String> {
    let officer = sim
        .crew
        .iter()
        .find(|member| member.archetype_id == post_id)
        .cloned()
        .ok_or_else(|| "A vacant post cannot teach an apprentice.".to_owned())?;
    if sim.apprenticeships.iter().any(|a| a.post_id == post_id) {
        return Err("That post already has a designated apprentice.".to_owned());
    }
    let cost = data.config.crew.apprentice_cost_credits;
    if sim.resources.credits < cost {
        return Err("The treasury cannot fund that apprenticeship.".to_owned());
    }
    sim.resources.credits -= cost;
    let given = sim
        .rng
        .choose(&data.dynasty_names.given_names)
        .cloned()
        .unwrap_or_else(|| "Unnamed".to_owned());
    let surname = data
        .dynasty_names
        .surnames_by_legacy
        .get(&sim.legacy.legacy_id)
        .and_then(|names| sim.rng.choose(names))
        .cloned()
        .unwrap_or_else(|| "Voyager".to_owned());
    let apprentice_name = format!("{given} {surname}");
    let skill = ((officer.skill as f32 * data.config.crew.apprentice_skill_retention).round()
        as u32)
        .max(1);
    sim.apprenticeships.push(Apprenticeship {
        post_id: post_id.to_owned(),
        apprentice_name: apprentice_name.clone(),
        faction_id: officer.faction_id,
        skill,
        designated_year: sim.year(),
    });
    sim.push_log(format!(
        "The council funds {apprentice_name} to learn the {post_id} craft."
    ));
    Ok(apprentice_name)
}

pub fn establish_or_support_school(
    sim: &mut SimState,
    data: &GameData,
    subsystem_id: &str,
) -> Result<String, String> {
    if data.subsystems.get(subsystem_id).is_none() {
        return Err("No such discipline exists.".to_owned());
    }
    let existing = sim
        .subsystem_schools
        .iter()
        .position(|school| school.subsystem_id == subsystem_id);
    let cost = if existing.is_some() {
        data.config.crew.school_upkeep_credits
    } else {
        data.config.crew.school_cost_credits
    };
    if sim.resources.credits < cost {
        return Err("The treasury cannot support that school.".to_owned());
    }
    sim.resources.credits -= cost;
    let until = sim.year() + data.config.crew.school_support_years;
    let name = data
        .subsystems
        .get(subsystem_id)
        .map_or(subsystem_id, |definition| definition.name.as_str());
    if let Some(index) = existing {
        sim.subsystem_schools[index].supported_until_year = until;
        sim.push_log(format!("The {name} school is funded through year {until}."));
    } else {
        sim.subsystem_schools.push(SubsystemSchool {
            subsystem_id: subsystem_id.to_owned(),
            founded_year: sim.year(),
            supported_until_year: until,
            custodian_faction_id: None,
        });
        sim.institution_records.push(InstitutionRecord {
            year: sim.year(),
            kind: InstitutionRecordKind::SchoolFounded,
            subject: format!("{name} school"),
            discipline: subsystem_id.to_owned(),
            knowledge_change: 0.0,
        });
        sim.push_log(format!("The council establishes a school for {name}."));
    }
    Ok(name.to_owned())
}

pub fn compile_archive(
    sim: &mut SimState,
    data: &GameData,
    subsystem_id: &str,
) -> Result<String, String> {
    if sim
        .procedure_archives
        .iter()
        .any(|archive| archive.subsystem_id == subsystem_id)
    {
        return Err("That discipline already has an emergency archive.".to_owned());
    }
    let cost = data.config.crew.archive_cost_credits;
    if sim.resources.credits < cost {
        return Err("The treasury cannot compile that archive.".to_owned());
    }
    let name = data
        .subsystems
        .get(subsystem_id)
        .map_or(subsystem_id, |definition| definition.name.as_str())
        .to_owned();
    sim.resources.credits -= cost;
    sim.procedure_archives.push(ProcedureArchive {
        subsystem_id: subsystem_id.to_owned(),
        compiled_year: sim.year(),
        preserved_experts: Vec::new(),
    });
    sim.institution_records.push(InstitutionRecord {
        year: sim.year(),
        kind: InstitutionRecordKind::ArchiveCompiled,
        subject: format!("{name} procedures"),
        discipline: subsystem_id.to_owned(),
        knowledge_change: 0.0,
    });
    sim.push_log(format!(
        "Emergency procedures for {name} enter the archive."
    ));
    Ok(name)
}

pub fn grant_custodianship(
    sim: &mut SimState,
    data: &GameData,
    subsystem_id: &str,
    faction_id: &str,
) -> Result<String, String> {
    let faction_aboard = sim
        .factions
        .iter()
        .any(|f| f.is_aboard() && f.faction_id == faction_id);
    if !faction_aboard {
        return Err("That people is no longer aboard to accept custody.".to_owned());
    }
    let influence = data.config.crew.custody_influence_cost;
    if sim.resources.influence < influence {
        return Err("The council lacks the influence to grant custody.".to_owned());
    }
    let school = sim
        .subsystem_schools
        .iter_mut()
        .find(|s| s.subsystem_id == subsystem_id)
        .ok_or_else(|| "Establish a school before granting custody.".to_owned())?;
    if school.custodian_faction_id.is_some() {
        return Err("That discipline already has a custodian.".to_owned());
    }
    school.custodian_faction_id = Some(faction_id.to_owned());
    sim.resources.influence -= influence;
    if let Some(faction) = sim
        .factions
        .iter_mut()
        .find(|f| f.faction_id == faction_id && f.is_aboard())
    {
        faction.adjust_approval(data.config.crew.custody_approval_gain);
    }
    let faction_name = data
        .factions
        .get(faction_id)
        .map_or(faction_id, |f| f.name.as_str())
        .to_owned();
    sim.institution_records.push(InstitutionRecord {
        year: sim.year(),
        kind: InstitutionRecordKind::CustodianshipGranted,
        subject: faction_name.clone(),
        discipline: subsystem_id.to_owned(),
        knowledge_change: 0.0,
    });
    sim.push_log(format!(
        "The {faction_name} receive custody of the {subsystem_id} discipline—and a stronger voice in council."
    ));
    Ok(faction_name)
}

pub fn officer_departed(sim: &mut SimState, data: &GameData, officer: &CrewMember) {
    let discipline = discipline_for_post(&officer.archetype_id).to_owned();
    let apprentice = sim
        .apprenticeships
        .iter()
        .position(|a| a.post_id == officer.archetype_id)
        .map(|index| sim.apprenticeships.remove(index));
    let school_active = sim.subsystem_schools.iter().any(|school| {
        school.subsystem_id == discipline && school.supported_until_year >= sim.year()
    });
    let archive_index = sim
        .procedure_archives
        .iter()
        .position(|archive| archive.subsystem_id == discipline);
    let mut loss = data.config.crew.unplanned_knowledge_loss;
    if school_active {
        loss *= 1.0 - data.config.crew.school_decay_reduction;
    }
    if archive_index.is_some() {
        loss *= 1.0 - data.config.crew.archive_loss_reduction;
    }
    if apprentice.is_some() {
        loss *= 1.0 - data.config.crew.apprentice_skill_retention;
    }
    if let Some(state) = sim.subsystems.get_mut(&discipline) {
        state.knowledge = (state.knowledge - loss).max(0.0);
    }
    if let Some(index) = archive_index {
        sim.procedure_archives[index]
            .preserved_experts
            .push(officer.name.clone());
    }
    let kind = if loss + f32::EPSILON < data.config.crew.unplanned_knowledge_loss {
        InstitutionRecordKind::ExpertisePreserved
    } else {
        InstitutionRecordKind::ExpertiseLost
    };
    sim.institution_records.push(InstitutionRecord {
        year: sim.year(),
        kind,
        subject: officer.name.clone(),
        discipline: discipline.clone(),
        knowledge_change: -loss,
    });

    if let Some(apprentice) = apprentice {
        let successor = CrewMember {
            id: sim.next_crew_id,
            name: apprentice.apprentice_name,
            archetype_id: apprentice.post_id,
            age: 25,
            skill: apprentice.skill,
            faction_id: apprentice.faction_id,
        };
        sim.next_crew_id += 1;
        sim.institution_records.push(InstitutionRecord {
            year: sim.year(),
            kind: InstitutionRecordKind::Appointment,
            subject: successor.name.clone(),
            discipline,
            knowledge_change: 0.0,
        });
        sim.push_log(format!(
            "{} succeeds {} with part of the old craft intact.",
            successor.name, officer.name
        ));
        sim.crew.push(successor);
    } else {
        sim.push_log(format!(
            "No successor was prepared for {}; {:.0}% of the related craft is lost.",
            officer.name,
            loss * 100.0
        ));
    }
}

#[cfg(test)]
mod tests;
