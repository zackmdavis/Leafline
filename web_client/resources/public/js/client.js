const Team = Object.freeze({Orange: "Orange", Blue: "Blue"});

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
}

let world = new WorldState();


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

function sendPostcard(news) {
    $message.text('');
    $.ajax({
        url: "/write/",
        method: 'POST',
        data: {
            world: world.preserve(),
        },
        success: function (data, textStatus, jqxhr) {
            let [newField, _initiative,
                 newEligibilities] = data.world.split(/ /);
            world.cedeInitiative();
            world.multifield.position(newField);
            world.preservedServiceEligibilities = newEligibilities;
            let commentary = ` (after ${data.thinking_time} ms thinking time)`;
            $spinner.hide();
            printHeadline(
                "Blue",
                data.patch.star,
                localeToAlgebraic(data.patch.whence),
                localeToAlgebraic(data.patch.whither),
                data.hospitalization,
                commentary
            );
            transpireYear();
        },
        error: function (jqxhr, textStatus, errorThrown) {
            $spinner.hide();
            $message.text(errorThrown);
        }
    });
}

function dropHandler(whence, whither, agentRune,
                     news, previously, _orientation) {
    if (whence != whither && whither != 'offboard') {
        let [team, opposition] = teamAndOppositionFromPrefix(agentRune[0]);
        let occupyingWhither = previously[whither];
        let patient;
        if (occupyingWhither) {
            let [patientTeam, _patientJobDescription] = occupyingWhither;
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

const $history = $('#history');
const $message = $('#message');
const $spinner = $('#spinner');

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
}
