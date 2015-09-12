#[macro_use]
macro_rules! moral_panic {
    ($y:expr) => (panic!("{}, which is contrary to the operation of the moral law!", $y))
}

#[macro_use]
macro_rules! match_agent {
    ( $agent:expr, $($team:ident, $job:ident => $val:expr),* ) => {
        match $agent {
            $( Agent { team: Team::$team,
                       job_description: JobDescription::$job } => $val ),*
        }
    }
}
