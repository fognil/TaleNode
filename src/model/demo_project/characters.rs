#[cfg(test)]
use crate::model::character::Character;
#[cfg(test)]
use crate::model::relationship::Relationship;

/// Holds character references so act builders can access IDs and names.
#[cfg(test)]
pub struct CharacterIds {
    pub voss: Character,
    pub elara: Character,
    pub kael: Character,
    pub thrix: Character,
    pub aria: Character,
    pub zeph: Character,
    pub saya: Character,
}

#[cfg(test)]
impl CharacterIds {
    pub fn all_cloned(&self) -> Vec<Character> {
        vec![
            self.voss.clone(),
            self.elara.clone(),
            self.kael.clone(),
            self.thrix.clone(),
            self.aria.clone(),
            self.zeph.clone(),
            self.saya.clone(),
        ]
    }

}

#[cfg(test)]
pub fn build_characters() -> CharacterIds {
    let mut voss = Character::new("Commander Voss");
    voss.color = [120, 144, 180, 255];
    voss.portrait_path = "portraits/voss.png".to_string();
    voss.relationships = vec![
        Relationship::new("Trust"),
        Relationship::new("Respect"),
    ];

    let mut elara = Character::new("Dr. Elara Chen");
    elara.color = [180, 220, 140, 255];
    elara.portrait_path = "portraits/elara.png".to_string();
    elara.relationships = vec![Relationship::new("Trust")];

    let mut kael = Character::new("Ambassador Kael");
    kael.color = [220, 190, 80, 255];
    kael.portrait_path = "portraits/kael.png".to_string();
    kael.relationships = vec![Relationship::new("Loyalty")];

    let mut thrix = Character::new("Warden Thrix");
    thrix.color = [160, 80, 180, 255];
    thrix.portrait_path = "portraits/thrix.png".to_string();

    let mut aria = Character::new("ARIA");
    aria.color = [100, 200, 220, 255];
    aria.portrait_path = "portraits/aria.png".to_string();
    aria.voice_id = Some("aria_synthetic_v2".to_string());

    let mut zeph = Character::new("Zeph");
    zeph.color = [220, 140, 60, 255];
    zeph.portrait_path = "portraits/zeph.png".to_string();

    let mut saya = Character::new("Elder Saya");
    saya.color = [200, 200, 200, 255];
    saya.portrait_path = "portraits/saya.png".to_string();

    CharacterIds { voss, elara, kael, thrix, aria, zeph, saya }
}
