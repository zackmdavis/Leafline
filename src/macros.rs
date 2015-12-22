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

#[macro_use]
macro_rules! assert_eq_within_ε {
    // crude edit of the canonical `assert_eq!`
    ($left:expr, $right:expr, $ε:expr) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if (*left_val - *right_val).abs() > $ε {
                    panic!("assertion failed: left and right not within ε \
                           (left: `{:?}`, right: `{:?}`)", left_val, right_val)
                }
            }
        }
    })
}
