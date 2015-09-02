use std::fmt;

use ansi_term::Colour as Color;  // this is America


#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub enum Team { Orange, Blue }

impl Team {
    pub fn league() -> Vec<Self> {
        // TODO: figure out how to return an iterator directly rather
        // than a vector on which we must call `.iter`
        vec![Team::Orange, Team::Blue]
    }

    pub fn opposition(&self) -> Self {
        match self {
            &Team::Orange => Team::Blue,
            &Team::Blue => Team::Orange
        }
    }
}

#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub enum JobDescription {
    Servant,  // ♂
    Pony,  // ♀
    Scholar,  // ♀
    Cop,  // ♂
    Princess,  // ♀
    Figurehead  // ♂
}

#[derive(Eq,PartialEq,Debug,Copy,Clone,Hash)]
pub struct Agent {
    pub team: Team,
    pub job_description: JobDescription
}

impl Agent {
    // I wanted to call it `dramatis_personæ`, but "non-ascii idents
    // are not fully supported" 🙀
    pub fn dramatis_personae(team: Team) -> Vec<Agent> {
        // TODO: return in iterator
        vec![Agent{ team: team,
                    job_description: JobDescription::Servant },
             Agent{ team: team,
                    job_description: JobDescription::Pony },
             Agent{ team: team,
                    job_description: JobDescription::Scholar },
             Agent{ team: team,
                    job_description: JobDescription::Cop },
             Agent{ team: team,
                    job_description: JobDescription::Princess },
             Agent{ team: team,
                    job_description: JobDescription::Figurehead }]
    }

    pub fn to_preservation_rune(&self) -> char {
        // XXX wow
        //      such boilerplate
        //   many agents     very repeat
        //                  wow
        match self {
            &Agent { team: Team::Orange,
                    // 'P' is for "peon"
                    job_description: JobDescription::Servant } => 'P',
            &Agent { team: Team::Orange,
                    // 'N' is for "neigh"
                    job_description: JobDescription::Pony } => 'N',
            &Agent { team: Team::Orange,
                    // 'B' is for "book"
                    job_description: JobDescription::Scholar } => 'B',
            &Agent { team: Team::Orange,
                    // 'R' is for "the Rule of law"
                    job_description: JobDescription::Cop } => 'R',
            &Agent { team: Team::Orange,
                    // 'Q' is the Princess's favorite letter of the alphabet
                    job_description: JobDescription::Princess } => 'Q',
            &Agent { team: Team::Orange,
                    // 'K' in baseball notation indicates a strikeout,
                    // which is bad; if the figurehead is in critical
                    // endangerment, his team loses the game, which is
                    // also bad
                    job_description: JobDescription::Figurehead } => 'K',
            // Blue Team's preservation runes are like Orange Team's
            // except in lowercase; this is because lowercase characters
            // have higher ASCII values, just as blue light has a higher
            // frequency than orange light
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Servant } => 'p',
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Pony } => 'n',
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Scholar } => 'b',
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Cop } => 'r',
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Princess } => 'q',
            &Agent { team: Team::Blue,
                    job_description: JobDescription::Figurehead } => 'k',
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let caricature = match self {
            &Agent { team: Team::Orange, .. } => {
                match self.job_description {
                    JobDescription::Servant => Color::Yellow.paint("♙"),
                    JobDescription::Pony => Color::Yellow.paint("♘"),
                    JobDescription::Scholar => Color::Yellow.paint("♗"),
                    JobDescription::Cop => Color::Yellow.paint("♖"),
                    JobDescription::Princess => Color::Yellow.paint("♕"),
                    JobDescription::Figurehead => Color::Yellow.paint("♔"),
                }
            },
            &Agent { team: Team::Blue, .. } => {
                match self.job_description {
                    JobDescription::Servant => Color::Cyan.paint("♟"),
                    JobDescription::Pony => Color::Cyan.paint("♞"),
                    JobDescription::Scholar => Color::Cyan.paint("♝"),
                    JobDescription::Cop => Color::Cyan.paint("♜"),
                    JobDescription::Princess => Color::Cyan.paint("♛"),
                    JobDescription::Figurehead => Color::Cyan.paint("♚"),
                }
            }
        };
        write!(f, "{}", caricature)
    }
}
