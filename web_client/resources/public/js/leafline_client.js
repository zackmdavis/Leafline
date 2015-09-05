function sendPostcard(_from, _to, _figurine, news, _previously, _orientation) {
    console.log(`sending postcard about ${news}`);
    $.ajax({
        url: "/write/",
        method: 'POST',
        data: {
            world: ChessBoard.objToFen(news),
        },
        success: function (data, textStatus, jqxhr) {
            console.log(`received response ${data}`);
            world.position(data.world);
        }
    });
}

const configuration = {
    position: 'start',
    draggable: true,
    pieceTheme: 'img/figurines/{piece}.png',
    onDrop: sendPostcard,
};
let world = ChessBoard('world', configuration);
