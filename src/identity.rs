use std::fmt;

use ansi_term::Colour as Color;  // this is America
use ansi_term::Style;

use serde::Serialize;



#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,Serialize)]
pub enum Team {
    Orange,
    Blue,
}

impl Team {
    pub fn league() -> Vec<Self> {
        // TODO: figure out how to return an iterator directly rather
        // than a vector on which we must call `.iter`
        vec![Team::Orange, Team::Blue]
    }

    pub fn opposition(&self) -> Self {
        match *self {
            Team::Orange => Team::Blue,
            Team::Blue => Team::Orange,
        }
    }

    pub fn figurine_paintjob(&self) -> Style {
        match *self {
            Team::Orange => Color::Red.bold(),
            Team::Blue => Color::Cyan.normal(),
        }
    }
}

#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,Serialize)]
pub enum JobDescription {
    Servant,  // ♂
    Pony,  // ♀
    Scholar,  // ♀
    Cop,  // ♂
    Princess,  // ♀
    Figurehead, // ♂
}

#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash,Serialize)]
pub struct Agent {
    pub team: Team,
    pub job_description: JobDescription,
}

static ORANGE_AGENTS: [Agent; 6] = [
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Servant
    },
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Pony
    },
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Scholar
    },
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Cop
    },
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Princess
    },
    Agent {
        team: Team::Orange,
        job_description: JobDescription::Figurehead
    },
];


static BLUE_AGENTS: [Agent; 6] = [
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Servant
    },
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Pony
    },
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Scholar
    },
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Cop
    },
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Princess
    },
    Agent {
        team: Team::Blue,
        job_description: JobDescription::Figurehead
    },
];

impl Agent {
    #![allow(clippy::wrong_self_convention)]
    pub fn new(team: Team, job_description: JobDescription) -> Self {
        Self { team, job_description }
    }

    pub fn dramatis_personæ(team: Team) -> [Agent; 6] {
        // TODO: return in iterator
        match team {
            Team::Orange => ORANGE_AGENTS,
            Team::Blue => BLUE_AGENTS
        }
    }

    pub fn to_preservation_rune(&self) -> char {
        match_agent!(*self,
            // 'P' is for "peon"
            Orange, Servant => 'P',
            // 'N' is for "neigh"
            Orange, Pony => 'N',
            // 'B' is for "book"
            Orange, Scholar => 'B',
            // 'R' is for "the Rule of law"
            Orange, Cop => 'R',
            // 'Q' is the Princess's favorite letter of the alphabet
            Orange, Princess => 'Q',
            // 'K' in baseball notation indicates a strikeout,
            // which is bad; if the figurehead is in critical
            // endangerment, his team loses the game, which is
            // also bad
            Orange, Figurehead => 'K',
            // Blue Team's preservation runes are like Orange Team's
            // except in lowercase; this is because lowercase characters
            // have higher ASCII values, just as blue light has a higher
            // frequency than orange light
            Blue,  Servant => 'p',
            Blue,  Pony => 'n',
            Blue,  Scholar => 'b',
            Blue,  Cop => 'r',
            Blue,  Princess => 'q',
            Blue,  Figurehead => 'k'
        )
    }

    pub fn to_pagan_movement_rune_prefix(&self) -> String {
        match self.job_description {
            JobDescription::Servant => "".to_owned(),
            // Pagan movement runes use the Orange job description forms for
            // nonservants
            _ => {
                format!("{}",
                        Agent::new(Team::Orange, self.job_description)
                            .to_preservation_rune())
            }
        }
    }

    pub fn to_figurine_display_rune(&self) -> char {
        match_agent!(*self,
            Orange, Servant => '♙',
            Orange, Pony => '♘',
            Orange, Scholar => '♗',
            Orange, Cop => '♖',
            Orange, Princess => '♕',
            Orange, Figurehead => '♔',
            Blue, Servant => '♟',
            Blue, Pony => '♞',
            Blue, Scholar => '♝',
            Blue, Cop => '♜',
            Blue, Princess => '♛',
            Blue, Figurehead => '♚'
         )
    }

    pub fn to_solid_display_rune(&self) -> char {
        // regrettably, the solid runes look better against locale
        // scenery, even though we would reserve them for Blue Team in
        // other contexts
        Agent::new(Team::Blue, self.job_description).to_figurine_display_rune()
    }
}


impl From<char> for Agent {
    fn from(rune: char) -> Self {
        match rune {
            'P' | '♙' => Agent::new(Team::Orange, JobDescription::Servant),
            'N' | '♘' => Agent::new(Team::Orange, JobDescription::Pony),
            'B' | '♗' => Agent::new(Team::Orange, JobDescription::Scholar),
            'R' | '♖' => Agent::new(Team::Orange, JobDescription::Cop),
            'Q' | '♕' => Agent::new(Team::Orange, JobDescription::Princess),
            'K' | '♔' => Agent::new(Team::Orange, JobDescription::Figurehead),
            'p' | '♟' => Agent::new(Team::Blue, JobDescription::Servant),
            'n' | '♞' => Agent::new(Team::Blue, JobDescription::Pony),
            'b' | '♝' => Agent::new(Team::Blue, JobDescription::Scholar),
            'r' | '♜' => Agent::new(Team::Blue, JobDescription::Cop),
            'q' | '♛' => Agent::new(Team::Blue, JobDescription::Princess),
            'k' | '♚' => Agent::new(Team::Blue, JobDescription::Figurehead),
            _ => {
                moral_panic!("tried to construct Agent from \
                              non-agent-preservation-rune (!?)")
            }
        }
    }
}


impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               self.team
                   .figurine_paintjob()
                   .paint(&self.to_figurine_display_rune().to_string()))
    }
}


#[cfg(test)]
mod tests {

    use encode;
    use super::Team;
    use identity::{JobDescription, Agent};

    #[test]
    fn test_team_serialization() {
        let o = Team::Orange;
        assert_eq!(r#""Orange""#, encode(&o));

        let b = Team::Blue;
        assert_eq!(r#""Blue""#, encode(&b));
    }

    #[test]
    fn test_job_description_serialization() {

        let servant = JobDescription::Servant;
        assert_eq!(r#""Servant""#, encode(&servant));


        let pony = JobDescription::Pony;
        assert_eq!(r#""Pony""#, encode(&pony));


        let scholar = JobDescription::Scholar;
        assert_eq!(r#""Scholar""#, encode(&scholar));


        let cop = JobDescription::Cop;
        assert_eq!(r#""Cop""#, encode(&cop));


        let princess = JobDescription::Princess;
        assert_eq!(r#""Princess""#, encode(&princess));


        let figurehead = JobDescription::Figurehead;
        assert_eq!(r#""Figurehead""#, encode(&figurehead));
    }

    #[test]
    fn test_agent_serialization() {
        let a = Agent {
            team: Team::Orange,
            job_description: JobDescription::Cop
        };

        assert_eq!( r#"{"team":"Orange","job_description":"Cop"}"#, encode(&a));
    }
}