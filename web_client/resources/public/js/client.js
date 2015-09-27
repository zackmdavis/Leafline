const Team = Object.freeze({Orange: "Orange", Blue: "Blue"});

// XXX hideous; can we put this somewhere else, please?
const initialReplies = [{star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 0}, whither: {rank: 2, file: 0}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 0}, whither: {rank: 3, file: 0}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 1}, whither: {rank: 2, file: 1}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 1}, whither: {rank: 3, file: 1}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 2}, whither: {rank: 2, file: 2}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 2}, whither: {rank: 3, file: 2}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 3}, whither: {rank: 2, file: 3}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 3}, whither: {rank: 3, file: 3}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 4}, whither: {rank: 2, file: 4}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 4}, whither: {rank: 3, file: 4}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 5}, whither: {rank: 2, file: 5}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 5}, whither: {rank: 3, file: 5}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 6}, whither: {rank: 2, file: 6}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 6}, whither: {rank: 3, file: 6}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 7}, whither: {rank: 2, file: 7}},  {star: {team: Team.Orange, job_description: "Servant"}, whence: {rank: 1, file: 7}, whither: {rank: 3, file: 7}},  {star: {team: Team.Orange, job_description: "Pony"}, whence: {rank: 0, file: 1}, whither: {rank: 2, file: 0}},  {star: {team: Team.Orange, job_description: "Pony"}, whence: {rank: 0, file: 1}, whither: {rank: 2, file: 2}},  {star: {team: Team.Orange, job_description: "Pony"}, whence: {rank: 0, file: 6}, whither: {rank: 2, file: 5}}, {star: {team: Team.Orange, job_description: "Pony"}, whence: {rank: 0, file: 6}, whither: {rank: 2, file: 7}}];

const configuration = {
    position: 'start',
    draggable: true,
    pieceTheme: 'img/figurines/{piece}.png',
    onDrop: dropHandler,
};

class WorldState {
    constructor() {
        this.initiative = Team.Orange;
        this.preservedServiceEligibilities = "KQkq";
        this.multifield = ChessBoard('world', configuration);
        this.replies = initialReplies;
    }

    preserveInitiative() {
        if (this.initiative === Team.Orange) {
            return 'w';
        } else if (this.initiative === Team.Blue) {
            return 'b';
        }
    }

    preserve() {
        return `${this.multifield.fen()} ` +
            `${this.preserveInitiative()} ` +
            `${this.preservedServiceEligibilities}`;
    }

    cedeInitiative() {
        this.initiative = (this.initiative === Team.Orange) ? Team.Blue : Team.Orange;
    }

    loadReplies(replies) {
        this.replies = replies;
    }

    validateMovement(movement) {
        let validations = (for (reply of this.replies)
            (_.isEqual(movement, reply)));
        for (var validity of validations) {
            if (validity) {
                return true;
            }
        }
        return false;
    }
}

let world = new WorldState();

const $history = $('#history');
const $message = $('#message');
const $spinner = $('#spinner');

const $ascensionModal = $('#ascension-modal')
const $ponyAscensionButton = $('#pony-ascension-button');
const $scholarAscensionButton = $('#scholar-ascension-button');
const $copAscensionButton = $('#cop-ascension-button');
const $princessAscensionButton = $('#princess-ascension-button');
let pendingAscension = null;

const $blueTriumphModal = $('#blue-triumph-modal');
const $orangeTriumphModal = $('#orange-triumph-modal');
const $deadlockModal = $('#deadlock-modal');

function leaflineAgentToGuiAgentRune(agent) {
    let teamToPrefix = {'Orange': "w", 'Blue': "b"};
    let jobDescriptionToTail = {
        'Servant': "P",
        'Pony': "N",
        'Scholar': "B",
        'Cop': "R",
        'Princess': "Q",
        'Figurehead': "K",
    };
    return (teamToPrefix[agent.team] +
            jobDescriptionToTail[agent.job_description]);
}

function guiAgentRuneToLeaflineAgent(rune) {
    let prefixToTeam = {'w': Team.Orange, 'b': Team.Blue};
    let tailToJobDescription = {
        'P': "Servant",
        'N': "Pony",
        'B': "Scholar",
        'R': "Cop",
        'Q': "Princess",
        'K': "Figurehead",
    };
    let [prefix, tail] = rune;
    return {'team': prefixToTeam[prefix],
            'job_description': tailToJobDescription[tail]};
}

function teamAndOppositionFromPrefix(prefix) {
    if (prefix === 'w') {
        return [Team.Orange, Team.Blue];
    } else if (prefix === 'b') {
        return [Team.Blue, Team.Orange];
    }
}

function localeToAlgebraic(locale) {
    let rank = String.fromCharCode(locale.rank + 49);  // 49 == '1'
    let file = String.fromCharCode(locale.file + 97);  // 97 == 'a'
    return file + rank;
}

function algebraicToLocale(algebraic) {
    let [fileIndicator, rankIndicator] = algebraic;
    return {rank: rankIndicator.charCodeAt() - 49,
            file: fileIndicator.charCodeAt() - 97};
}

function getLookaheadBound() {
    let nature;
    if ($('#depth-radio-button').is(':checked')) {
        nature = "depth";
    } else if ($('#seconds-radio-button').is(':checked')) {
        nature = "seconds";
    } else {
        throw "need to select a lookahead bound–nature"
    }
    let value = parseInt($('#bound-input').val());
    return { nature: nature, value: value }
}


function sendPostcard(news) {
    $message.removeClass("alert");
    $message.text('');
    $.ajax({
        url: "/write/",
        method: 'POST',
        data: {
            world: world.preserve(),
            bound: getLookaheadBound()
        },
        success: function (missive, textStatus, jqxhr) {
            if ("the_triumphant" in missive) {
                if (missive.the_triumphant === Team.Orange) {
                    $orangeTriumphModal.foundation('reveal', "open");
                } else if (missive.the_triumphant === Team.Blue) {
                    $blueTriumphModal.foundation('reveal', "open");
                } else {
                    $deadlockModal.foundation('reveal', "open");
                }
                $spinner.hide();
                return;
            }
            let [newField, _initiative,
                 newEligibilities] = missive.world.split(/ /);
            world.cedeInitiative();
            world.multifield.position(newField);
            world.preservedServiceEligibilities = newEligibilities;
            world.replies = missive.counterreplies;
            let commentary = ` (after searching ${missive.depth} plies in ` +
                             `${missive.thinking_time} ms)`;
            $spinner.hide();
            printHeadline(
                "Blue",
                missive.patch.star,
                localeToAlgebraic(missive.patch.whence),
                localeToAlgebraic(missive.patch.whither),
                missive.hospitalization,
                commentary
            );
            transpireYear();
        },
        error: function (jqxhr, textStatus, errorThrown) {
            $spinner.hide();
            $message.addClass("alert").addClass("alert-box");
            $message.text(`${errorThrown}: ${jqxhr.responseJSON.error}`);
        }
    });
}


function dropHandler(whence, whither, agentRune,
                     news, previously, _orientation) {
    if (whence != whither && whither != 'offboard') {
        let [team, _opposition] = teamAndOppositionFromPrefix(agentRune[0]);
        let agent = guiAgentRuneToLeaflineAgent(agentRune);
        let movement = {star: agent,
                        whence: algebraicToLocale(whence),
                        whither: algebraicToLocale(whither)};
        if (!world.validateMovement(movement)) {
            return "snapback";
        }
        if (agent.job_description === "Servant" && movement.whither.rank === 7) {
            // ascension
            $ascensionModal.foundation('reveal', "open");
            pendingAscension = movement;
            return "snapback";
        }
        if (agent.job_description === "Figurehead" &&
            Math.abs(movement.whence.file - movement.whither.file) === 2) {
            // secret service
            //
            // XXX TODO (here as elsewhere): eventually we might want to
            // drop the assumption that the human controls the Orange
            // Team
            let longitude = movement.whither.file - movement.whence.file;
            if (longitude === 2) {  // east service
                delete news['h1'];
                news.f1 = "wR";
            } else if (longitude === -2) {  // west service
                delete news['a1'];
                news.d1 = "wR";
            }
            world.preservedServiceEligibilities = world
                .preservedServiceEligibilities.replace(/Q/g, '');
            world.preservedServiceEligibilities = world
                .preservedServiceEligibilities.replace(/K/g, '');
        }
        let occupyingWhither = previously[whither];
        let patient;
        if (occupyingWhither) {
            patient = guiAgentRuneToLeaflineAgent(occupyingWhither)
        } else {
            patient = null;
        }
        world.multifield.position(news, false);
        world.cedeInitiative();
        sendPostcard(news);
        $spinner.show();
        printHeadline(
            "Orange", guiAgentRuneToLeaflineAgent(agentRune),
            whence, whither, patient, null
        );
    }
}

function getYear() {
    return parseInt($history.attr('data-year'));
}

function transpireYear() {
    $history.attr('data-year', getYear() + 1);
}


function guiAgentRuneToIcon(agentRune) {
    return $('<img>').attr('src', `img/figurines/${agentRune}.png`)
        .addClass('agent-icon');
}

function leaflineAgentToIcon(agent) {
    return guiAgentRuneToIcon(leaflineAgentToGuiAgentRune(agent));
}

function printHeadline(team, agent, whence, whither,
                       hospitalization, commentary) {
    let year = getYear();
    let $headline = $('<div />').addClass("headline")
        .attr('data-team', team).attr('data-year', year);
    let dateline;
    if (team === "Orange") {
        dateline = year + ". ";
    } else if (team === "Blue") {
        dateline = ".. ";
    }
    $headline.append($('<strong />').text(dateline));

    $headline.append(leaflineAgentToIcon(agent));
    let newsEvent = ` from ${whence} to ${whither}`;
    $headline.append($('<span />').text(newsEvent));

    if (hospitalization) {
        let hospitalMugshot = leaflineAgentToIcon(hospitalization);
        $headline.append(
            $('<span />').text(`, stunning `)
        );
        $headline.append(hospitalMugshot);
    }

    if (commentary) {
        $headline.append($('<em />').text(commentary).addClass("commentary"));
    }

    $history.append($headline);
    $history.scrollTop($history[0].scrollHeight);
}

$(document).ready(function() {
    $('#ascension-modal-close-control').on('click', function (_event) {
        pendingAscension = null;
    });
    $('.ascension-button').on('click', function (_event) {
        // XXX factorize
        if (world.validateMovement(pendingAscension)) {
            $ascensionModal.foundation('reveal', "close");
            let ascendedFormRune = `w${$(this).data('job-description-rune')}`;
            window.setTimeout(function () {
                let news = world.multifield.position();
                let algebraicWhence = localeToAlgebraic(pendingAscension.whence);
                let algebraicWhither = localeToAlgebraic(
                    pendingAscension.whither);
                delete news[algebraicWhence];
                news[algebraicWhither] = ascendedFormRune;
                world.multifield.position(news);
                world.cedeInitiative();
                printHeadline(
                    "Orange", guiAgentRuneToLeaflineAgent(ascendedFormRune),
                    algebraicWhence, algebraicWhither,
                    null, // XXX need to take stunning ascension into account
                    " (servant ascension)"
                );
                pendingAscension = null;
                sendPostcard(news);
                $spinner.show();
            }, 500);
        } else {
            alert("No Pending Ascension Error");  // yeah, that just happened
        }
    });
});
