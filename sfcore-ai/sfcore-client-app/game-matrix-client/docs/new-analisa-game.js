$(document).ready(async function () {
    await delayedGreeting();
    await getLogResult();
    await generateTablePickUp();
    await checkQueueIsReadyOrNot();
    await tambahNote('GET');
});

async function generateTablePickUp() {
    var dt = [];
    var no = 0;
    var listAngka = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09"];
    for (let index = 10; index < 100; index++) {
        listAngka.push(index.toString());
    }

    var tbl = $('.tblpicker>tbody');
    for (var a = 0; a < listAngka.length; a++) {
        dt.push(listAngka[a]);
        if (no === 9) {
            var row = "<tr>";
            for (var i = 0; i < dt.length; i++) {
                row = row + "<td><button onclick='notPickNumber(\"" + dt[i] + "\")' tite='click on here for not pick number " + dt[i] + "' class='btn btn-lg btn-xs btn-pck btpx-" + dt[i] + "'>" + dt[i] + "</button></td>";
            }
            row = row + "</tr>";
            no = 0;
            dt = [];
            tbl.append(row);
        }
        else {
            no++;
        }
    }
}

window.setNumberNotInGame = async function (tipe) {

    await setPickNumberIn(tipe);
    if (tipe == 'AKK-X') {
        let numb = $('#Tb3DA').val();
        if (numb.length == 0 || numb == '') {
            alert('THE ANGKA BELUM DI SET UNTUK DI MATIKAN, HARAP PILIH ANGKA TERLEBIH DAHULU.');
            return false;
        } else {
            await notPickNumber(numb);
        }
    }
    else if (tipe == 'AKE-X') {
        let numb = $('#Tb3DB').val();
        if (numb.length == 0 || numb == '') {
            alert('THE ANGKA BELUM DI SET UNTUK DI MATIKAN, HARAP PILIH ANGKA TERLEBIH DAHULU.');
            return false;
        } else {
            await notPickNumber(numb);
        }
    }
    else if (tipe == 'APE-X') {
        let numb = $('#Tb3DC').val();
        if (numb.length == 0 || numb == '') {
            alert('THE ANGKA BELUM DI SET UNTUK DI MATIKAN, HARAP PILIH ANGKA TERLEBIH DAHULU.');
            return false;
        } else {
            await notPickNumber(numb);
        }
    }
    else if (tipe == 'ALL-X') {
        let numb = $('#Tb3DC').val();
        await notPickNumber(numb);

        numb = $('#Tb3DD').val();
        await notPickNumber(numb);
        numb = $('#Tb3DB').val();
        await notPickNumber(numb);
        numb = $('#Tb3DA').val();
        await notPickNumber(numb);
    }
    else if (tipe == 'FB-X') {
        let numb = $('#Tb3DA').val();
        await notPickNumber(numb);

        numb = $('#Tb3DD').val();
        await notPickNumber(numb);
    }
    else if (tipe == 'FBA-X') {
        let numb = $('#Tb3DA').val();
        await notPickNumber(numb);

        numb = $('#Tb3DD').val();
        await notPickNumber(numb);
        numb = $('#Tb3DC').val();
        await notPickNumber(numb);
    }
    else {
        let numb = $('#Tb3DD').val();
        if (numb.length == 0 || numb == '') {
            alert('THE ANGKA BELUM DI SET UNTUK DI MATIKAN, HARAP PILIH ANGKA TERLEBIH DAHULU.');
            return false;
        } else {
            await notPickNumber(numb);
        }
    }
}
window.notPickNumber = async function (numb) {
    var tipe = $('#hidTipeSet').val();
    if (tipe === "0") {
        alert('tipe set belum di pilih, harap pilih terlebih dahulu');
        return false;
    }
    let typePick = "2D-F";
    var bgcolor = '#0d6efd';
    if (tipe === '2dm') { bgcolor = '#ffc107'; typePick = "2D-M"; }
    else if (tipe === '2db') { bgcolor = 'green'; typePick = "2D-B"; }
    else if (tipe === 'AK') { bgcolor = '#0dcaf0'; typePick = "AS-KEPALA"; }
    else if (tipe === 'AE') { bgcolor = 'blueviolet'; typePick = "AS-EKOR"; }
    else if (tipe === 'KE') { bgcolor = '#dc3545'; typePick = "KOP-EKOR"; }
    else if (tipe === 'AKK-X') { bgcolor = '#dc3545'; typePick = "AS-KOP-KEP"; }
    else if (tipe === 'AKE-X') { bgcolor = '#dc3545'; typePick = "AS-KOP-EKOR"; }
    else if (tipe === 'APE-X') { bgcolor = '#dc3545'; typePick = "AS-KEP-EKOR"; }
    else if (tipe === 'KKE-X') { bgcolor = '#dc3545'; typePick = "KOP-KEP-EKOR"; }

    let stat = true;
    await showToastr('Set Angka ' + numb + ' ini Mati.? untuk tipe : ' + typePick, 'SET NOT PICK NUMBER');
    if (stat) {
        await delayedGreeting();

        $('.btpx-' + numb).css("background-color", bgcolor);
        var gameCode = $('#hidCode').val();
        var gameId = $('#hidId').val();
        var vm = {
            GameId: gameId,
            GameCode: gameCode,
            TyppePick: typePick,
            TheNumber: numb
        };

        $.ajax({
            type: "POST",
            url: "/ApiQueue/SaveAnalisa",
            data: JSON.stringify(vm),
            contentType: "application/json; charset=utf-8",
            dataType: "json",
            success: async function (data) {
                await showToastr('Set Angka ' + numb + ' ini Mati.? untuk tipe : ' + typePick + ' [SUKSES]', 'SET NOT PICK NUMBER', 'FROM SERVER');
            },
            error: function (req, status, error) {
                console.log(error);
            }
        });

    }
    //let digitOne = numb.substr(0, 1);
    //let digitTwo = numb.substr(1, 1);

}

async function checkQueueIsReadyOrNot() {
    var code = $('#hidCode').val();
    $.ajax({
        type: 'GET',
        url: '/ApiQueue/CheckQue/' + code,
        success: async function (data) {
            if (data == "1" || data === 1) {
                $('.dv-info-que').css('display', 'block');
            }
        }
    });
}

async function getLogResult() {
    sessionStorage.clear();
    var code = $('#hidCode').val();
    $.ajax({
        type: 'GET',
        url: '/ApiHomeAnalisa/GetLogResult/' + code,
        success: async function (data) {
            await populateDataLogTable(data.logGameDtos);
            //await populatePrediksi(data.prediksiDtos);
            //await populatePrediksiDenganTipeSama(data.prediksiSameTipeDtos);
            sessionStorage.setItem('dsLogGame', JSON.stringify(data.logGameDtos));
            //console.log(data.logGameDtos);
        }
    });
}

async function populateDataLogTable(data) {

    var tbl = $('.tbl-log>tbody');
    tbl.empty();
    var dt = [];
    var no = 0;
    var baris = 0;
    let currentLog;
    for (var a = 0; a < data.length; a++) {
        if (a == 0) currentLog = data[0];
        dt.push({ periode: data[a].periode, log: data[a].logResult, as: data[a].as, kop: data[a].kop, kepala: data[a].kepala, ekor: data[a].ekor });
        if (no === 6) {
            var row = "<tr>";
            for (var i = 0; i < dt.length; i++) {
                let borderColor = checkLogIfSameData(currentLog, dt[i]);
                if ((baris == 3 && i == 0) || (baris == 8 && i == 0)) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if ((baris == 4 && i == 1) || (baris == 9 && i == 1)) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if (baris == 5 && i == 2) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if (baris == 6 && i == 3) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if (baris == 7 && i == 4) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if (baris == 8 && i == 5) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
                else if (baris == 9 && i == 6) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                    baris = baris - 5;
                }
                else {
                    row = row + "<td title='" + dt[i].periode + "' style='cursor:pointer;" + borderColor + "' class='tdp-" + dt[i].periode + "' onclick='changeBackground(\"" + dt[i].periode + "\")'>" + dt[i].log + "</td>";
                }
            }
            row = row + "</tr>";
            tbl.append(row);
            no = 0;
            dt = [];
            await delayedDom();
            baris++;
        }
        else {
            no++;
        }

        if (a > 120) {
            break;
        }
    }


    await closeProgressBar();
    //console.log(currentLog);
    // for (let i = 2; i < 10; i++) {
    //     await populateAnotherLogTable(data, i.toString())
    // }
}

window.checkLogIfSameData = function (currentLog, compareLog) {


    if (currentLog.as == compareLog.as && currentLog.kop == compareLog.kop) {
        return "border:2px solid orangered";
    }
    else if (currentLog.as == compareLog.kop && currentLog.kop == compareLog.kepala) {
        return "border-bottom:2px solid orangered";
    }
    else if (currentLog.as == compareLog.kepala && currentLog.kop == compareLog.ekor) {
        return "border-bottom:2px solid orangered";
    }
    else if (currentLog.kop == compareLog.as && currentLog.kepala == compareLog.kop) {
        return "border-bottom:2px solid yellow";
    }
    else if (currentLog.kop == compareLog.kop && currentLog.kepala == compareLog.kepala) {
        return "border:2px solid yellow";
    }
    else if (currentLog.kop == compareLog.kepala && currentLog.kepala == compareLog.ekor) {
        return "border-bottom:2px solid yellow";
    }
    else if (currentLog.kepala == compareLog.as && currentLog.ekor == compareLog.kop) {
        return "border-bottom:2px solid green";
    }
    else if (currentLog.kepala == compareLog.kop && currentLog.ekor == compareLog.kepala) {
        return "border-bottom:2px solid green";
    }
    else if (currentLog.kepala == compareLog.kepala && currentLog.ekor == compareLog.ekor) {
        return "border:2px solid green";
    }
    else {
        return "";
    }
}

window.changeBackground = async function (idx, numbInfo) {

    if (numbInfo == undefined || numbInfo == null) {
        if ($('.tdp-' + idx).css("background-color") === "rgba(0, 0, 0, 0)") { $('.tdp-' + idx).css("background-color", "rgb(232, 157, 192)"); }
        else if ($('.tdp-' + idx).css("background-color") === "rgb(232, 157, 192)") { $('.tdp-' + idx).css("background-color", "rgb(136, 205, 208)"); }
        else if ($('.tdp-' + idx).css("background-color") === "rgb(136, 205, 208)") { $('.tdp-' + idx).css("background-color", "rgb(221, 222, 222)"); }
        else if ($('.tdp-' + idx).css("background-color") === "rgb(221, 222, 222)") { $('.tdp-' + idx).css("background-color", "rgb(198, 246, 210)"); }
        else if ($('.tdp-' + idx).css("background-color") === "rgb(198, 246, 210)") { $('.tdp-' + idx).css("background-color", "rgb(198, 220, 246)"); }
        else { $('.tdp-' + idx).css("background-color", "rgb(0, 0, 0, 0)"); }

        //--proses pencarian data yang sama berdasarkan Periode...
        let ds = sessionStorage.getItem('dsLogGame');
        ds = JSON.parse(ds);

        let selectedLog = ds.find(x => x.periode == parseInt(idx));
        $('.label-info-result').text(selectedLog.logResult);
        let digitFronts = [];
        let digitMids = [];
        let digitBacks = [];
        for await (const log of ds) {
            if (selectedLog.as == log.as && selectedLog.kop == log.kop) digitFronts.push(log);
            if (selectedLog.kop == log.kop && selectedLog.kepala == log.kepala) digitMids.push(log);
            if (selectedLog.kepala == log.kepala && selectedLog.ekor == log.ekor) digitBacks.push(log);
        }

        /*function for get list by front and mid and back.......*/


        let newListFronts = [];
        digitFronts.forEach(log => {
            let prevPeriode = log.periode - 1;
            let nextPeriode = log.periode + 1;

            let prevLog = ds.find(x => x.periode == prevPeriode);
            let nextLog = ds.find(x => x.periode == nextPeriode);
            let listLogs = [];
            listLogs.push(prevLog);
            listLogs.push(log);
            listLogs.push(nextLog);
            let vm = { periode: log.periode, logs: listLogs };
            newListFronts.push(vm);
        });

        let newListMids = [];
        digitMids.forEach(log => {
            let prevPeriode = log.periode - 1;
            let nextPeriode = log.periode + 1;

            let prevLog = ds.find(x => x.periode == prevPeriode);
            let nextLog = ds.find(x => x.periode == nextPeriode);
            let listLogs = [];
            listLogs.push(prevLog);
            listLogs.push(log);
            listLogs.push(nextLog);
            let vm = { periode: log.periode, logs: listLogs };
            newListMids.push(vm);
        });

        let newListBacks = [];
        digitBacks.forEach(log => {
            let prevPeriode = log.periode - 1;
            let nextPeriode = log.periode + 1;

            let prevLog = ds.find(x => x.periode == prevPeriode);
            let nextLog = ds.find(x => x.periode == nextPeriode);
            let listLogs = [];
            listLogs.push(prevLog);
            listLogs.push(log);
            listLogs.push(nextLog);
            let vm = { periode: log.periode, logs: listLogs };
            newListBacks.push(vm);
        });

        let divParent = $('.dv-same-log');
        divParent.empty();
        let row = "<div class='col-md-4' style='border:1px solid crimson;border-radius:10px'>";
        row += "<table class='table table-sm table-border table-hover'><thead><tr><th>P</th><th>C</th><th>N</th></tr></thead><tbody>";

        let urutan = 1;
        for await (const log of newListFronts) {
            try {
                if (log.periode < parseInt(idx)) {
                    row += "<tr>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[0].periode + "\",\"" + log.logs[0].logResult + "\")' class='tdpxx-" + log.logs[0].periode + " td-rpf' data-index='" + urutan + "' >" + log.logs[0].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[1].periode + "\",\"" + log.logs[1].logResult + "\")' class='tdpxx-" + log.logs[1].periode + " td-rcf' data-index='" + urutan + "' style='border:1px solid crimson;'>" + log.logs[1].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[2].periode + "\",\"" + log.logs[2].logResult + "\")' class='tdpxx-" + log.logs[2].periode + " td-rbf' data-index='" + urutan + "' >" + log.logs[2].logResult + "</td>";
                    row += "<tr>";
                    urutan++;
                }
            } catch (error) {
                console.log('error try catch');
                console.log(error);
            }
        }
        row += "</tbody></table></div>";
        divParent.append(row);
        row = '';

        row = "<div class='col-md-4' style='border:1px solid orange;border-radius:10px'>";
        row += "<table class='table table-sm table-border table-hover'><thead><tr><th>P</th><th>C</th><th>N</th></tr></thead><tbody>";
        urutan = 1;
        for await (const log of newListMids) {
            try {
                if (log.periode < parseInt(idx)) {
                    row += "<tr>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[0].periode + "\",\"" + log.logs[0].logResult + "\")' class='tdpxx-" + log.logs[0].periode + " td-rpm' data-index='" + urutan + "' >" + log.logs[0].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[1].periode + "\",\"" + log.logs[1].logResult + "\")' class='tdpxx-" + log.logs[1].periode + " td-rcm' data-index='" + urutan + "'  style='border:1px solid #e1cf11;'>" + log.logs[1].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[2].periode + "\",\"" + log.logs[2].logResult + "\")' class='tdpxx-" + log.logs[2].periode + " td-rbm' data-index='" + urutan + "' >" + log.logs[2].logResult + "</td>";
                    row += "<tr>";
                }

            } catch (error) {
                console.log('error try catch');
                console.log(error);

            }
        }
        row += "</tbody></table></div>";
        divParent.append(row);

        row = '';

        row = "<div class='col-md-4'style='border:1px solid green;border-radius:10px'>";
        row += "<table class='table table-sm table-border table-hover'><thead><tr><th>P</th><th>C</th><th>N</th></tr></thead><tbody>";
        urutan = 1;

        for await (const log of newListBacks) {
            try {
                if (log.periode < parseInt(idx)) {
                    row += "<tr>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[0].periode + "\",\"" + log.logs[0].logResult + "\")' class='tdpxx-" + log.logs[0].periode + " td-rpb' data-index='" + urutan + "' >" + log.logs[0].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[1].periode + "\",\"" + log.logs[1].logResult + "\")' class='tdpxx-" + log.logs[1].periode + " td-rcb' data-index='" + urutan + "'  style='border:1px solid green;'>" + log.logs[1].logResult + "</td>";
                    row += "<td onclick='changeBgSelectedData(\"" + log.logs[2].periode + "\",\"" + log.logs[2].logResult + "\")' class='tdpxx-" + log.logs[2].periode + " td-rbb' data-index='" + urutan + "' >" + log.logs[2].logResult + "</td>";
                    row += "<tr>";
                }

            } catch (error) {
                console.log('error try catch');
                console.log(error);
            }
        }
        row += "</tbody></table></div>";
        divParent.append(row);

        let dvbox = $('.dvbox-simulate-pick>table');
        if (dvbox.length == 0) {
            let contentx = buildTableSimulatePickupNumber();
            $('.dvbox-simulate-pick').append(contentx);
        } else {
            $('.btn-simulate-pick').each(function (a, b) {
                $(b).css('background', '#ffffff');
                $(b).css('color', '#333');
                $(b).attr('data-color', '');
            });
        }


        //-- get Analisa Pattern Result...
        await loadAnalisisPatternResult();
        await generateAnalisaPatternL(newListFronts, newListMids, newListBacks);
        await getDataListByTwoDigitNumberSelected(ds,selectedLog);
    }
    else {
        var logResult = $(idx).text();
        if ($(idx).css("background-color") === "rgba(0, 0, 0, 0)") { $(idx).css("background-color", "rgb(232, 157, 192)"); copyDataToHitTextBox(logResult);}
        else if ($(idx).css("background-color") === "rgb(232, 157, 192)") { $(idx).css("background-color", "rgb(136, 205, 208)"); copyDataToHitTextBox(logResult); }
        else if ($(idx).css("background-color") === "rgb(136, 205, 208)") { $(idx).css("background-color", "rgb(221, 222, 222)");  copyDataToHitTextBox(logResult);}
        else if ($(idx).css("background-color") === "rgb(221, 222, 222)") { $(idx).css("background-color", "rgb(198, 246, 210)");  copyDataToHitTextBox(logResult);}
        else if ($(idx).css("background-color") === "rgb(198, 246, 210)") { $(idx).css("background-color", "rgb(198, 220, 246)");  copyDataToHitTextBox(logResult);}
        else { $(idx).css("background-color", "rgb(0, 0, 0, 0)"); }

    }

}
function copyDataToHitTextBox(log) {
    var asx = log.substring(0, 1);
    var kop = log.substring(1, 2);
    var kepala = log.substring(2, 3);
    var ekor = log.substring(3, 4);

    $('#Tb3DA').val(asx + '-' + kop + '-' + kepala);
    $('#Tb3DB').val(asx + '-' + kop + '-' + ekor);
    $('#Tb3DC').val(asx + '-' + kepala + '-' + ekor);
    $('#Tb3DD').val(kop + '-' + kepala + '-' + ekor);


    $('#Tb3DA-2').val(asx + '-' + kop + '-' + kepala);
    $('#Tb3DB-2').val(asx + '-' + kop + '-' + ekor);
    $('#Tb3DC-2').val(asx + '-' + kepala + '-' + ekor);
    $('#Tb3DD-2').val(kop + '-' + kepala + '-' + ekor);

}


window.getDataListByTwoDigitNumberSelected = async function (ds,selectedLog) {

    let listNumberByFront = [];
    let listNumberByMid = [];
    let listNumberByBack = [];

    let isFound = false;
    let logByFront = [];
    for await (const log of ds) {
        if(!isFound) {
            if(log.periode < selectedLog.periode) {
                if (selectedLog.as == log.as && selectedLog.kop == log.kop || selectedLog.as == log.kop && selectedLog.kop == log.kepala || selectedLog.as == log.kepala && selectedLog.kop == log.ekor ) {
                    isFound = true;
                    logByFront.push(log);
                }
            }
        }
    }

    isFound = false;
    let logByMid = [];
    for await (const log of ds) {
        if(!isFound) {
            if(log.periode < selectedLog.periode) {
                if (selectedLog.kop == log.as && selectedLog.kepala == log.kop || selectedLog.kop == log.kop && selectedLog.kepala == log.kepala || selectedLog.kop == log.kepala && selectedLog.kepala == log.ekor ) {
                    isFound = true;
                    logByMid.push(log);
                }
            }
        }
    }

    isFound = false;
    let logByBack = [];
    for await (const log of ds) {
        if(!isFound) {
            if(log.periode < selectedLog.periode) {
                if (selectedLog.kepala == log.as && selectedLog.ekor == log.kop || selectedLog.kepala == log.kop && selectedLog.ekor == log.kepala || selectedLog.kepala == log.kepala && selectedLog.ekor == log.ekor ) {
                    isFound = true;
                    logByBack.push(log);
                }
            }
        }
    }


    let PeriodeFront =  logByFront[0].periode - 10;
    for (let a = 0; a < 21 ; a++) {
        let log = ds.find(x=>x.periode == PeriodeFront);
        PeriodeFront++;        

        if(log != undefined){
            listNumberByFront.push(log);
        }
        else {
            listNumberByFront.push({ as : 0, createdDate:null,dateResultInGame:null,ekor:0,gameCode: '-',kepala:0,kop:0,logResult:'-',periode:PeriodeFront });
        }
    }


    let PeriodeMid =  logByMid[0].periode - 10;
    for (let a = 0; a < 21 ; a++) {
        let log = ds.find(x=>x.periode == PeriodeMid);
        PeriodeMid++;        

        if(log != undefined){
            listNumberByMid.push(log);
        }
        else {
            listNumberByMid.push({ as : 0, createdDate:null,dateResultInGame:null,ekor:0,gameCode: '-',kepala:0,kop:0,logResult:'-',periode:PeriodeMid });
        }
    }

    let PeriodeBack =  logByBack[0].periode - 10;
    for (let a = 0; a < 21 ; a++) {
        let log = ds.find(x=>x.periode == PeriodeBack);
        PeriodeBack++;        

        if(log != undefined){
            listNumberByBack.push(log);
        }
        else {
            listNumberByBack.push({ as : 0, createdDate:null,dateResultInGame:null,ekor:0,gameCode: '-',kepala:0,kop:0,logResult:'-',periode:PeriodeBack });
        }
    }

    let tbl = $('#tbl-sd1>tbody');
    tbl.empty();

    listNumberByFront.sort((a, b) => b.periode - a.periode);
    for (let i = 0; i < listNumberByFront.length; i++) {
        const log = listNumberByFront[i];
        let row = '<tr>';
        if(i == 10) {
            row += '<td style="cursor:pointer;background-color:#dedede" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        } else {
            row += '<td style="cursor:pointer" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        }
        row += '</tr>';
        tbl.append(row);
    }

    tbl = $('#tbl-sd2>tbody');
    tbl.empty();

    listNumberByMid.sort((a, b) => b.periode - a.periode);
    for (let i = 0; i < listNumberByMid.length; i++) {
        const log = listNumberByMid[i];
        let row = '<tr>';
        if(i == 10) {
            row += '<td style="cursor:pointer;background-color:#dedede" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        } else {
            row += '<td style="cursor:pointer" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        }
        row += '</tr>';
        tbl.append(row);
    }

    tbl = $('#tbl-sd3>tbody');
    tbl.empty();

    listNumberByBack.sort((a, b) => b.periode - a.periode);
    for (let i = 0; i < listNumberByBack.length; i++) {
        const log = listNumberByBack[i];
        let row = '<tr>';
        if(i == 10) {
            row += '<td style="cursor:pointer;background-color:#dedede;" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        } else {
            row += '<td style="cursor:pointer" onclick="changeBgSelectedData(\''+log.periode+'\' , \'' +log.logResult+ '\', this)">'+log.logResult+'</td>';
        }
        row += '</tr>';
        tbl.append(row);
    }
}



window.setPickNumberIn = async function (tipe) {
    if (tipe === '2df') { $('.sp-set').text('SET 2D FRONT') }
    else if (tipe === '2dm') {
        $('.sp-set').text('SET 2D MID');
    }
    else if (tipe === '2db') { $('.sp-set').text('SET 2D BACK'); }
    else if (tipe === 'AK') { $('.sp-set').text('SET AS + KEPALA'); }
    else if (tipe === 'AE') { $('.sp-set').text('SET AS + EKOR'); }
    else if (tipe === 'KE') { $('.sp-set').text('SET KOP + EKOR'); }
    else if (tipe === 'AKK-X') { $('.sp-set').text('SET AS + KOP + KEPALA (3D-FRONT)'); }
    else if (tipe === 'AKE-X') { $('.sp-set').text('SET AS + KOP + EKOR'); }
    else if (tipe === 'APE-X') { $('.sp-set').text('SET AS + KEPALA + EKOR'); }
    else if (tipe === 'KKE-X') { $('.sp-set').text('SET KOP + KEPALA + EKOR (3D-BACK)'); }

    $('#hidTipeSet').val(tipe);
}


window.changeBgSelectedData = async function (idx, log, elm) {
    if ($('.tdpxx-' + idx).css("background-color") === "rgba(0, 0, 0, 0)") { $('.tdpxx-' + idx).css("background-color", "rgb(232, 157, 192)"); }
    else if ($('.tdpxx-' + idx).css("background-color") === "rgb(232, 157, 192)") { $('.tdpxx-' + idx).css("background-color", "rgb(136, 205, 208)"); }
    else if ($('.tdpxx-' + idx).css("background-color") === "rgb(136, 205, 208)") { $('.tdpxx-' + idx).css("background-color", "rgb(221, 222, 222)"); }
    else if ($('.tdpxx-' + idx).css("background-color") === "rgb(221, 222, 222)") { $('.tdpxx-' + idx).css("background-color", "rgb(198, 246, 210)"); }
    else if ($('.tdpxx-' + idx).css("background-color") === "rgb(198, 246, 210)") { $('.tdpxx-' + idx).css("background-color", "rgb(198, 220, 246)"); }
    else { $('.tdpxx-' + idx).css("background-color", "rgb(0, 0, 0, 0)"); }

    var asx = log.substring(0, 1);
    var kop = log.substring(1, 2);
    var kepala = log.substring(2, 3);
    var ekor = log.substring(3, 4);

    let A1 = asx.toString() + kop.toString();
    let A2 = kop.toString() + asx.toString();
    let A3 = kop.toString() + kepala.toString();
    let A4 = kepala.toString() + kop.toString();
    let A5 = kepala.toString() + ekor.toString();
    let A6 = ekor.toString() + kepala.toString();

    let patternNumbers = [];
    patternNumbers.push(parseInt(A1));
    patternNumbers.push(parseInt(A2));
    patternNumbers.push(parseInt(A3));
    patternNumbers.push(parseInt(A4));
    patternNumbers.push(parseInt(A5));
    patternNumbers.push(parseInt(A6));

    let selectedNumbers = [];
    patternNumbers.forEach(patt => {
        let originalNumber = patt;

        let plusOne = patt + 1;
        if (plusOne == 100) plusOne = 0;

        let minusOne = patt - 1;
        if (minusOne == -1) minusOne = 99;

        let plusTen = patt + 10;
        if (plusTen >= 100) plusTen -= 100;

        let minTen = patt - 10;
        if (minTen < 0) minTen += 100;

        let plusElevent = 0;
        if (originalNumber == 99) plusElevent = 80;
        else plusElevent = patt + 11;

        if (plusElevent > 99) plusElevent -= 100;

        let plusNine = patt + 9;
        if (plusNine > 99) plusNine -= 100;

        let minElevent = patt - 11;
        if (minElevent < 0) minElevent += 100;

        let minNine = patt - 9;
        if (minNine < 0) minNine += 100;

        if (originalNumber == 9 || originalNumber == 19 || originalNumber == 29 || originalNumber == 39 || originalNumber == 49 || originalNumber == 59 || originalNumber == 69
            || originalNumber == 79 || originalNumber == 89 || originalNumber == 99) {
            let minusSembilanBelas = patt - 19;
            selectedNumbers.push(minusSembilanBelas);
        }


        selectedNumbers.push(originalNumber);
        selectedNumbers.push(plusOne);
        selectedNumbers.push(minusOne);
        selectedNumbers.push(plusTen);
        selectedNumbers.push(plusElevent);
        selectedNumbers.push(plusNine);
        selectedNumbers.push(minTen);
        selectedNumbers.push(minElevent);
        selectedNumbers.push(minNine);

    });

    let newNumbers = [];
    selectedNumbers.forEach(pat => {
        let numb = "";
        if (pat.toString().length == 1) numb = "0" + pat.toString();
        else numb = pat.toString();
        newNumbers.push(numb);
    });

    for (let a = 0; a < newNumbers.length; a++) {
        const numb = newNumbers[a];
        let domId = '#btn-simulate-' + numb;
        if ($(domId).attr('data-color') == "ciyan") {
            $(domId).css('background', 'lime');
            $(domId).attr('data-color', 'lime');
        }
        else if ($(domId).attr('data-color') == "lime") {
            $(domId).css('background', 'yellowgreen');
            $(domId).attr('data-color', 'yellowgreen');
        }
        else if ($(domId).attr('data-color') == "yellowgreen") {
            $(domId).attr('data-color', 'green');
            $(domId).css('background', 'green');
        }
        else if ($(domId).attr('data-color') == "green") {
            $(domId).css('background', 'darkgreen');
            $(domId).css('color', '#ffffff');
            $(domId).attr('data-color', 'darkgreen');
        }
        else if ($(domId).attr('data-color') == "darkgreen") {
            $(domId).css('color', 'yellow');
            $(domId).attr('data-color', '#023b02');
        }
        else if ($(domId).attr('data-color') == "#023b02") {
            $(domId).css('background', '#023b02');
            $(domId).css('color', 'crimson');
            $(domId).attr('data-color', 'darkgreen');
        }
        else {
            $(domId).css('background', 'lightcyan');
            $(domId).attr('data-color', 'ciyan');
        }
    }
    //---1. 

    //-- set text-box..
    $('#Tb3DA').val(asx + '-' + kop + '-' + kepala);
    $('#Tb3DB').val(asx + '-' + kop + '-' + ekor);
    $('#Tb3DC').val(asx + '-' + kepala + '-' + ekor);
    $('#Tb3DD').val(kop + '-' + kepala + '-' + ekor);


    $('#Tb3DA-2').val(asx + '-' + kop + '-' + kepala);
    $('#Tb3DB-2').val(asx + '-' + kop + '-' + ekor);
    $('#Tb3DC-2').val(asx + '-' + kepala + '-' + ekor);
    $('#Tb3DD-2').val(kop + '-' + kepala + '-' + ekor);

    if(elm != null || elm != undefined) {
        let colorx = $(elm).css('background-color');
        if(colorx === 'rgb(232, 165, 232)') {
            $(elm).css('background','rgb(33 179 170)');
        }else {
            $(elm).css('background','#e8a5e8');
        }
    }
}

window.loadAnalisisPatternResult = async function () {
    await delayedGreeting();
    var code = $('#hidCode').val();
    $.ajax({
        type: 'GET',
        url: '/ApiHomeAnalisa/GetAnalisisPatternResult/' + code,
        success: async function (data) {
            populateDataAnalisisPatternToTableSameData(data);
            await closeProgressBar();
        }
    });
}

window.populateDataAnalisisPatternToTableSameData = async function (data) {
    let ds = [];
    for await (const da of data) {
        if (da.typePattern == 'F') {
            ds.push(da);
        }
    }

    ds.forEach(log => {
        if (log.statusFlag.toString().includes('P')) {
            $('.td-rpf').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('C')) {
            $('.td-rcf').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('N')) {
            $('.td-rbf').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
    });


    for await (const da of data) {
        if (da.typePattern == 'M') {
            ds.push(da);
        }
    }

    ds.forEach(log => {
        if (log.statusFlag.toString().includes('P')) {
            $('.td-rpm').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('C')) {
            $('.td-rcm').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('N')) {
            $('.td-rbm').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
    });



    for await (const da of data) {
        if (da.typePattern == 'B') {
            ds.push(da);
        }
    }

    ds.forEach(log => {
        if (log.statusFlag.toString().includes('P')) {
            $('.td-rpb').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('C')) {
            $('.td-rcb').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
        else if (log.statusFlag.toString().includes('N')) {
            $('.td-rbb').each(function (a, b) {
                let di = $(b).attr('data-index');
                if (di == log.indexSameResult.toString()) {
                    $(b).css('background-color', 'rgb(236 230 255)');
                }
            });
        }
    });

}
function rangeNumberPick(nomor) {
    let one = "";

}

function buildTableSimulatePickupNumber() {
    var dt = [];
    var no = 0;
    var listAngka = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09"];
    for (let index = 10; index < 100; index++) {
        listAngka.push(index.toString());
    }

    var row = "<table><tbody>";
    for (var a = 0; a < listAngka.length; a++) {
        dt.push(listAngka[a]);
        if (no === 9) {
            row += "<tr>";
            for (var i = 0; i < dt.length; i++) {
                row = row + "<td><button class='btn btn-pck btn-md btn-simulate-pick' data-color='' id='btn-simulate-" + dt[i] + "'>" + dt[i] + "</button></td>";
            }
            row = row + "</tr>";
            no = 0;
            dt = [];
        }
        else {
            no++;
        }
    }
    row = row + "</tbody></table>";
    return row;
}

window.generateAnalisaPatternL = async function (newListFronts, newListMids, newListBacks) {

}

window.generateAnalisaMini = async function () {
    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);

    let logs = [];
    let newdsMini = [];
    for (let i = 0; i < ds.length; i++) {
        if (i < 366) {
            const logx = ds[i];
            let nexp = parseInt(logx.periode) + 1;
            let nexr = ds.find(x => x.periode == nexp);
            logs.push({ log: logx, fs: [], ms: [], bs: [], np: nexr });
            newdsMini.push(logx);
        } else {
            break;
        }
    }

    let newLogs = [];

    for (let i = 0; i < logs.length; i++) {
        try {
            const logx = logs[i];

            var fss = ds.find(x => x.as == logx.log.as && x.kop == logx.log.kop && x.periode < logx.log.periode);
            let prevPeriode = parseInt(fss.periode) - 1;
            let nextPeriode = parseInt(fss.periode) + 1;

            var fssp = ds.find(x => x.periode == prevPeriode);
            var fssn = ds.find(x => x.periode == nextPeriode);
            let datafs = [];
            datafs.push(fssp);
            datafs.push(fss);
            datafs.push(fssn);


            var mss = ds.find(x => x.kop == logx.log.kop && x.kepala == logx.log.kepala && x.periode < logx.log.periode);
            prevPeriode = parseInt(mss.periode) - 1;
            nextPeriode = parseInt(mss.periode) + 1;

            fssp = ds.find(x => x.periode == prevPeriode);
            fssn = ds.find(x => x.periode == nextPeriode);
            let datams = [];
            datams.push(fssp);
            datams.push(mss);
            datams.push(fssn);


            var bss = ds.find(x => x.kepala == logx.log.kepala && x.ekor == logx.log.ekor && x.periode < logx.log.periode);
            prevPeriode = parseInt(bss.periode) - 1;
            nextPeriode = parseInt(bss.periode) + 1;

            fssp = ds.find(x => x.periode == prevPeriode);
            fssn = ds.find(x => x.periode == nextPeriode);
            let databs = [];
            databs.push(fssp);
            databs.push(bss);
            databs.push(fssn);


            let vm = { log: logx, fs: datafs, ms: datams, bs: databs, np: logx.np };

            newLogs.push(vm);

        } catch (error) {
            console.log(error);
        }
    }// -- end of for logs


    let dv = $('.dv-box-ruls');
    dv.empty();
    for (let i = 0; i < newLogs.length; i++) {
        const log = newLogs[i];

        try {

            if (log.np == undefined) log.np = { periode: 'xxxx', logResult: "XXXX" };

            var row = "<div class='row' style='margin-bottom:2px;border-bottom:1px solid orangered;text-align:center'>";

            row += "<div class='col-md-3' style='margin:1px;border:1px solid #666'>";
            row += "<table class='table table-border'>";
            row += "<thead>";
            row += "<tr>";
            row += "<th>P</th>";
            row += "<th>C</th>";
            row += "<th>N</th>";
            row += "</tr>";
            row += "</thead>";

            row += "<tbody>";
            row += "<tr>";
            row += "<td class='TD-F-P-XX-" + log.np.periode + "' onclick='rubahWarnaElemen(this)'>" + log.fs[0].logResult + "</td>";
            row += "<td class='TD-F-C-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'><b>" + log.fs[1].logResult + "</b></td>";
            row += "<td class='TD-F-N-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'>" + log.fs[2].logResult + "</td>";
            row += "</tr>";
            row += "</tbody>";
            row += "</table>";
            row += "</div>";


            row += "<div class='col-md-3' style='margin:1px;border:1px solid #666'>";
            row += "<table class='table table-border'>";
            row += "<thead>";
            row += "<tr>";
            row += "<th>P</th>";
            row += "<th>C</th>";
            row += "<th>N</th>";
            row += "</tr>";
            row += "</thead>";

            row += "<tbody>";
            row += "<tr>";
            row += "<td class='TD-M-P-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'>" + log.ms[0].logResult + "</td>";
            row += "<td class='TD-M-C-XX-" + log.np.periode + "'   onclick='rubahWarnaElemen(this)'><b>" + log.ms[1].logResult + "</b></td>";
            row += "<td class='TD-M-N-XX-" + log.np.periode + "'   onclick='rubahWarnaElemen(this)'>" + log.ms[2].logResult + "</td>";
            row += "</tr>";
            row += "</tbody>";
            row += "</table>";
            row += "</div>";

            row += "<div class='col-md-3' style='margin:1px;border:1px solid #666'>";
            row += "<table class='table table-border'>";
            row += "<thead>";
            row += "<tr>";
            row += "<th>P</th>";
            row += "<th>C</th>";
            row += "<th>N</th>";
            row += "</tr>";
            row += "</thead>";

            row += "<tbody>";
            row += "<tr>";
            row += "<td class='TD-B-P-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'>" + log.bs[0].logResult + "</td>";
            row += "<td class='TD-B-C-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'><b>" + log.bs[1].logResult + "</b></td>";
            row += "<td class='TD-B-N-XX-" + log.np.periode + "'  onclick='rubahWarnaElemen(this)'>" + log.bs[2].logResult + "</td>";
            row += "</tr>";
            row += "</tbody>";
            row += "</table>";
            row += "</div>";

            row += "<div class='col-md-2' style='margin:1px;border:1px solid purple'>";
            if (log.np == undefined) {
                row += "<b style='color:crimson;font-size:16px'> XXXX <= " + log.log.log.logResult + "</b><br/><br/>";
                row += "<span style='color:green;font-size:16px' id='bcount-x-" + log.np.periode + "-" + log.log.log.logResult + "'>0</span>";
            } else {
                row += "<b style='color:crimson;font-size:16px'>" + log.np.logResult + " <= " + log.log.log.logResult + "</b><br/><br/>";
                row += "<span style='color:green;font-size:16px' id='bcount-x-" + log.np.periode + "-" + log.log.log.logResult + "'>0</span>";
            }
            row += "</div>";


            row += "</div>";

            dv.append(row);
        } catch (error) {
            console.log(error);
        }

    }


    sessionStorage.setItem('dsAnxMini', JSON.stringify(newLogs));

    await populateAnotherLogTable(newdsMini, '989');
}

window.rubahWarnaElemen = function (elm) {
    let bgc = $(elm).css('background-color');
    //console.log(bgc);

    if (bgc == 'rgba(0, 0, 0, 0)') {
        $(elm).css('background', 'yellow');
        $(elm).css('color', '#000');
    }
    else if (bgc == 'rgb(255, 255, 0)') {
        $(elm).css('background', 'cadetblue');
        $(elm).css('color', '#fff');
    }
    else {
        $(elm).css('background', 'rgba(0, 0, 0, 0)');
        $(elm).css('color', '#000');
    }

}

window.populateAnotherLogTable = async function (data, count) {

    let countTable = '.tbl-log-' + count + '>tbody';
    var tbl = $(countTable);
    tbl.empty();
    var dt = [];
    var no = 0;
    var baris = 0;
    let currentLog;
    for (var a = 0; a < data.length; a++) {
        if (a == 0) currentLog = data[0];
        dt.push({ periode: data[a].periode, log: data[a].logResult, as: data[a].as, kop: data[a].kop, kepala: data[a].kepala, ekor: data[a].ekor });
        if (no === 6) {
            var row = "<tr>";
            for (var i = 0; i < dt.length; i++) {
                let borderColor = "";
                if ((baris == 3 && i == 0) || (baris == 8 && i == 0)) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if ((baris == 4 && i == 1) || (baris == 9 && i == 1)) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if (baris == 5 && i == 2) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if (baris == 6 && i == 3) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if (baris == 7 && i == 4) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if (baris == 8 && i == 5) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                }
                else if (baris == 9 && i == 6) {
                    row = row + "<td title='" + dt[i].periode + "'  style='cursor:pointer;background-color : rgb(236 230 255);" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                    baris = baris - 5;
                }
                else {
                    if(baris == 1 && i === 0 && count !== "2") {
                        row = row + "<td title='" + dt[i].periode + "' style='cursor:pointer;background-color : green;color:white;" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                    }
                    else 
                    {
                        row = row + "<td title='" + dt[i].periode + "' style='cursor:pointer;" + borderColor + "' class='tdp-" + count + "-xx-" + dt[i].periode + "' onclick='changeBackground(this,22)'>" + dt[i].log + "</td>";
                    }
                }
            }
            row = row + "</tr>";
            tbl.append(row);
            no = 0;
            dt = [];
            baris++;
        }
        else {
            no++;
        }
    }
}


window.generateHistoryMini = async function (tipe) {
    let dsAm = sessionStorage.getItem('dsAnxMini');
    dsAm = JSON.parse(dsAm);

    if (tipe == 'SF' || tipe == 'SB') {
        for (let i = 0; i < dsAm.length; i++) {
            try {
                if (i > 0) {
                    const data = dsAm[i];
                    const nextResult = data.np.logResult;
                    let digitResult = '';
                    if (tipe == 'SF') digitResult = nextResult.substring(0, 2);
                    else digitResult = nextResult.substring(2, 4);

                    let digitLog = '';
                    let selectorData = '.tdp-989-xx-' + data.log.log.periode;
                    let selectorLog = '.TD-B-P-XX-' + data.np.periode;

                    let countx = 0;
                    for await (const patts of data.fs) {
                        if (tipe == 'SF') digitLog = patts.logResult.substring(0, 2);
                        else digitLog = patts.logResult.substring(2, 4);

                        if (countx == 0) selectorLog = '.TD-F-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-F-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-F-N-XX-' + data.np.periode;

                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult);
                        countx++;
                    }

                    countx = 0;
                    for await (const patts of data.ms) {
                        if (tipe == 'SF') digitLog = patts.logResult.substring(0, 2);
                        else digitLog = patts.logResult.substring(2, 4);

                        if (countx == 0) selectorLog = '.TD-M-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-M-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-M-N-XX-' + data.np.periode;
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult);
                        countx++;
                    }

                    countx = 0;
                    for await (const patts of data.bs) {
                        if (tipe == 'SF') digitLog = patts.logResult.substring(0, 2);
                        else digitLog = patts.logResult.substring(2, 4);

                        if (countx == 0) selectorLog = '.TD-B-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-B-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-B-N-XX-' + data.np.periode;
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult);
                        countx++;
                    }
                    await sleep(13);

                }

            } catch (error) {
                console.log(error);
            }
        }
    }
    else if (tipe == 'SFALL' || tipe == 'SBALL') {
        for (let i = 0; i < dsAm.length; i++) {
            try {
                if (i > 0) {
                    const data = dsAm[i];
                    const nextResult = data.np.logResult;
                    let digitResult = '';
                    if (tipe == 'SFALL') digitResult = nextResult.substring(0, 2);
                    else digitResult = nextResult.substring(2, 4);

                    let digitLog = '';
                    let selectorData = '.tdp-989-xx-' + data.log.log.periode;
                    let selectorLog = '.TD-B-P-XX-' + data.np.periode;
                    let selectorCountx = '#bcount-x-' + data.np.periode + "-" + data.log.log.logResult;

                    let countx = 0;
                    for await (const patts of data.fs) {
                        digitLog = patts.logResult.substring(0, 2);

                        if (countx == 0) selectorLog = '.TD-F-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-F-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-F-N-XX-' + data.np.periode;

                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(1, 3);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(2, 4);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);

                        digitLog = patts.kop.toString() + patts.as.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.kepala.toString() + patts.kop.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.ekor.toString() + patts.kepala.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        countx++;
                    }

                    countx = 0;
                    for await (const patts of data.ms) {
                        digitLog = patts.logResult.substring(0, 2);

                        if (countx == 0) selectorLog = '.TD-M-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-M-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-M-N-XX-' + data.np.periode;
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(1, 3);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(2, 4);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);

                        digitLog = patts.kop.toString() + patts.as.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.kepala.toString() + patts.kop.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.ekor.toString() + patts.kepala.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        countx++;
                    }

                    countx = 0;
                    for await (const patts of data.bs) {
                        digitLog = patts.logResult.substring(0, 2);

                        if (countx == 0) selectorLog = '.TD-B-P-XX-' + data.np.periode;
                        else if (countx == 1) selectorLog = '.TD-B-C-XX-' + data.np.periode;
                        else selectorLog = '.TD-B-N-XX-' + data.np.periode;
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(1, 3);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.logResult.substring(2, 4);
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);

                        digitLog = patts.kop.toString() + patts.as.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.kepala.toString() + patts.kop.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        digitLog = patts.ekor.toString() + patts.kepala.toString();
                        await changeSelectorDataLog(tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx);
                        countx++;
                    }
                    await sleep(13);
                }
            }
            catch (error) {
                console.log(error);
            }
        }
    }
    else {
        for (let i = 0; i < dsAm.length; i++) {
            try {
                if (i > 0) {
                    const data = dsAm[i];
                    const nextResult = data.np.logResult;
                    let digitResult = '';
                    if (tipe == 'F') digitResult = nextResult.substring(0, 2);
                    else digitResult = nextResult.substring(2, 4);

                    //console.log(digitResult);

                    for await (const patts of data.fs) {
                        await findSelectedLog(patts.logResult, digitResult.toString(), data.log.log.periode, tipe);
                    }

                    for await (const patts of data.ms) {
                        await findSelectedLog(patts.logResult, digitResult.toString(), data.log.log.periode, tipe);
                    }

                    for await (const patts of data.bs) {
                        await findSelectedLog(patts.logResult, digitResult.toString(), data.log.log.periode, tipe);
                    }
                    await sleep(20);
                }

            } catch (error) {
                console.log(error);
            }
        }

    }
}

window.findSelectedLog = async function (log, digitResult, periode, tipe) {
    var asx = log.substring(0, 1);
    var kop = log.substring(1, 2);
    var kepala = log.substring(2, 3);
    var ekor = log.substring(3, 4);

    let A1 = asx.toString() + kop.toString();
    let A2 = kop.toString() + asx.toString();
    let A3 = kop.toString() + kepala.toString();
    let A4 = kepala.toString() + kop.toString();
    let A5 = kepala.toString() + ekor.toString();
    let A6 = ekor.toString() + kepala.toString();

    let patternNumbers = [];
    patternNumbers.push(parseInt(A1));
    patternNumbers.push(parseInt(A2));
    patternNumbers.push(parseInt(A3));
    patternNumbers.push(parseInt(A4));
    patternNumbers.push(parseInt(A5));
    patternNumbers.push(parseInt(A6));

    let selectedNumbers = [];
    patternNumbers.forEach(patt => {
        let originalNumber = patt;

        let plusOne = patt + 1;
        if (plusOne == 100) plusOne = 0;

        let minusOne = patt - 1;
        if (minusOne == -1) minusOne = 99;

        let plusTen = patt + 10;
        if (plusTen >= 100) plusTen -= 100;

        let minTen = patt - 10;
        if (minTen < 0) minTen += 100;

        let plusElevent = 0;
        if (originalNumber == 99) plusElevent = 80;
        else plusElevent = patt + 11;

        if (plusElevent > 99) plusElevent -= 100;

        let plusNine = patt + 9;
        if (plusNine > 99) plusNine -= 100;

        let minElevent = patt - 11;
        if (minElevent < 0) minElevent += 100;

        let minNine = patt - 9;
        if (minNine < 0) minNine += 100;

        if (originalNumber == 9 || originalNumber == 19 || originalNumber == 29 || originalNumber == 39 || originalNumber == 49 || originalNumber == 59 || originalNumber == 69
            || originalNumber == 79 || originalNumber == 89 || originalNumber == 99) {
            let minusSembilanBelas = patt - 19;
            selectedNumbers.push(minusSembilanBelas);
        }


        selectedNumbers.push(originalNumber);
        selectedNumbers.push(plusOne);
        selectedNumbers.push(minusOne);
        selectedNumbers.push(plusTen);
        selectedNumbers.push(plusElevent);
        selectedNumbers.push(plusNine);
        selectedNumbers.push(minTen);
        selectedNumbers.push(minElevent);
        selectedNumbers.push(minNine);

    });

    let newNumbers = [];
    selectedNumbers.forEach(pat => {
        let numb = "";
        if (pat.toString().length == 1) numb = "0" + pat.toString();
        else numb = pat.toString();
        newNumbers.push(numb);
    });

    let isSame = newNumbers.find(x => x == digitResult);
    if (isSame != undefined) {
        let selectorData = '.tdp-989-xx-' + periode;
        if (tipe == 'F') {
            $(selectorData).css('background', 'yellow');
            $(selectorData).css('color', '#333');
        } else {
            $(selectorData).css('background', 'cadetblue');
            $(selectorData).css('color', '#fff');
        }
    }

}

window.changeSelectorDataLog = function (tipe, selectorData, selectorLog, digitLog, digitResult, selectorCountx) {
    if (digitLog == digitResult) {
        let countx = $(selectorCountx).text();
        let cx = parseInt(countx);
        cx++;
        $(selectorCountx).text(cx);
        if (tipe == 'SF' || tipe == 'SFALL') {
            $(selectorData).css('background', 'orange');
            $(selectorData).css('color', '#333');
            $(selectorLog).css('background', 'orange');
            $(selectorLog).css('color', '#333');

        } else {
            $(selectorData).css('background', 'rgb(91 73 136)');
            $(selectorData).css('color', '#fff');
            $(selectorLog).css('background', 'rgb(91 73 136)');
            $(selectorLog).css('color', '#fff');
        }
    }
}

async function generateAnalisaFOrRobotTwoDigit() {
    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);

    let dataBacks = [];
    let jarak = 0;
    for (let i = 0; i < ds.length; i++) {
        const log = ds[i];
        let dback = (parseInt(log.kepala) * 10) + log.ekor;
        if (dback > 49) {
            jarak++;
        } else {
            if (jarak > 5) {
                let aa = { periode: log.periode, logResult: log.logResult, intensitas: jarak };
                dataBacks.push(aa);
            }
            jarak = 0;
        }
    }

    let div = $('#dv-robot-jarak');
    div.empty();
    let row = '<div class="col-md-2"><table class="table border"><tbody>';
    dataBacks.forEach(data => {
        row += '<tr>';
        row += '<td>' + data.periode + '</td>';
        row += '<td>' + data.logResult + '</td>';
        row += '<td>' + data.intensitas + '</td>';
        row += '</tr>';
    });
    row += '</tbody></table></div>';
    div.append(row);



}


async function callApiAnalysDataVertical() {
    var code = $('#hidCode').val();
    $.ajax({
        type: 'GET',
        url: '/ApiHitGame/GetLogAnalis/' + code,
        success: async function (data) {
            //console.log('--data analis vertical--');
            //console.log(data);
            await populateDataVertical(data);
            //console.log('--end of data analis vertical--');
        }
    });
}

async function populateDataVertical(data) {
    let tblHeader = $('.tblList-vertical');
    tblHeader.empty();
    let row = "<tr>";
    var nomor = 1;
    for (let a = 0; a < data.length; a++) {
        const log = data[a];
        row += "<td>";
        row += "<div class='col-anx' style='border:1px solid crimson'>";
        row += "<div class='col-lg-12'><span style='font-weight:500;font-size:14px;color:crimson'>"+log.logResultTemplate+"</span></div>";
        row += "<div class='col-lg-12' style='color:purple'>"+log.periode+" - <span style='color:purple;font-size:16px; font-weight:bold'>#"+nomor+"</span></div>";
        row += "<table class='table-analis-vertical' id='tbl-log-"+log.periode+"'>";
        row += "<tbody></tbody>";
        row += "</table>";
        row += "</div>";
        row += "</td>";
        nomor++;
    }
    row += "</tr>";
    tblHeader.append(row);
    sessionStorage.setItem('dsLogAnalisaVertical',JSON.stringify(data));
    /*bind Log Table*/

  
}

async function generateDataLogAnalisa() {
    let ds = sessionStorage.getItem('dsLogAnalisaVertical');
    var data = JSON.parse(ds);

    for (let i = 0; i < data.length; i++) {
        const log = data[i];
        let tblDetail = $('#tbl-log-' + log.periode +'>tbody');
        let rw = "";  
        for (let a = 0; a < 21; a++) {
            rw = "<tr>"; 

            try {
                const df = log.logTemplates.logFronts;
                if(log.logPosition.logFronts !=null ) { 
                    var isready = log.logPosition.logFronts.find(x=>x.periode == df[a].periode);
                    if(isready != undefined) {
                        rw += "<td style='background:green;color:#fff'>"+df[a].logResult+"</td>";
                    } else {
                        if(df[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+df[a].logResult+"</td>";
                        else rw += "<td>"+df[a].logResult+"</td>";
                    }
                }else {
                    if(df[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+df[a].logResult+"</td>";
                    else rw += "<td>"+df[a].logResult+"</td>";
            }    
            } catch(error) { 
                rw += "<td>-</td>";
            }

            try {
                const dm = log.logTemplates.logMids;
                if(log.logPosition.logMids !=null ) { 
                    var isready = log.logPosition.logMids.find(x=>x.periode == dm[a].periode);
                    if(isready != undefined) {
                        rw += "<td style='background:yellow;'>"+dm[a].logResult+"</td>";
                    } else {
                        if(dm[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+dm[a].logResult+"</td>";
                        else rw += "<td>"+dm[a].logResult+"</td>";
                    }
                }else {
                    if(dm[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+dm[a].logResult+"</td>";
                    else rw += "<td>"+dm[a].logResult+"</td>";
            }
            } catch(error) { 
                rw += "<td>-</td>";
            }

            try {
                const db = log.logTemplates.logBacks;
                if(log.logPosition.logBacks !=null ) { 
                    var isready = log.logPosition.logBacks.find(x=>x.periode == db[a].periode);
                    if(isready != undefined) {
                        rw += "<td style='background:crimson;'>"+db[a].logResult+"</td>";
                    } else {
                        if(db[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+db[a].logResult+"</td>";
                        else rw += "<td>"+db[a].logResult+"</td>";
                        }
                }else {
                    if(db[a].dateResultInGame == 'INX') rw += "<td style='background:#666;color:#fff'>"+db[a].logResult+"</td>";
                    else rw += "<td>"+db[a].logResult+"</td>";
                }
            } catch(error) { 
                rw += "<td>-</td>";
            }
            rw += "</tr>";
            tblDetail.append(rw);
            await sleep(10);
        }
    }
}

async function tambahNote(tipe){
    var code = $('#hidCode').val();
    if(tipe === 'save') {
        let note = $('#tbNote').val();
        let vm = {gameCode : code, note : note};
    
        $.ajax({
            type: "POST",
            url: "/ApiQueue/SaveNote",
            data: JSON.stringify(vm),
            contentType: "application/json; charset=utf-8",
            dataType: "json",
            success: async function (data) {
                await bindNotes(data);
            },
            error: function (req, status, error) {
                console.log(error);
            }
        });        
    }
    else {
        $.ajax({
            type: 'GET',
            url: '/ApiQueue/GetNote/' + code,
            success: async function (data) {
                bindNotes(data);
            },
            error : async function(error) {
                console.log('getting error get NOTE');
            }
        });
    }

    let bindNotes = async function(data){
        let divNote = $('#dvNote');
        divNote.empty();
        for (let a = 0; a < data.length; a++) {
            const note = data[a];
            let row = "<ul><i>*# </i>"+note.note+"</ul>";
            divNote.append(row);
        }
    }
}



async function findDataLogResult(){
    let logResult = $('.label-info-result').text();
    $('.lbl-new-infolog').text(logResult);

    var asx = logResult.substring(0, 1);
    var kop = logResult.substring(1, 2);
    var kepala = logResult.substring(2, 3);
    var ekor = logResult.substring(3, 4);

    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);
    let tempDsFront = [];
    let tempDsBack = [];

    for (let i = 0; i < ds.length; i++) {
        if(i !== 0) {
            const log = ds[i];

            if(log.as.toString() == asx && log.kop.toString() == kop && log.kepala.toString() == kepala ) {
                tempDsFront.push(log);
            }

            if(log.kop.toString() == kop && log.kepala.toString() == kepala && log.ekor.toString() == ekor) {
                tempDsBack.push(log);
            }

        }
    }

    let divAwal = $('#tblDataSameLog>tbody');
    divAwal.empty();

    //cetak data front awal...di table 
    for (let i = 0; i < tempDsBack.length; i++) {
        const element = tempDsBack[i];
        tempDsFront.push(element);        
    }

    let awalTabel = 3;
    for (let i = 0; i < tempDsFront.length; i++) {
        const element = tempDsFront[i];
        let row = "<tr>";
        row += "<td style='cursor:pointer' onclick='generateDataLogIni(\"" + element.periode + "\",\"" + awalTabel + "\")' >"+element.periode+"<td>";
        row += "<td style='cursor:pointer' onclick='generateDataLogIni(\"" + element.periode + "\",\"" + awalTabel + "\")' >"+element.logResult+"<td>";
        row += "</tr>";
        divAwal.append(row);
        awalTabel++;
    }

    let dsLogPrimitif = (ds.slice(0,365));

    // Buat 7 dummy object
    const dummyData = Array.from({ length: 7 }, (_, i) => ({
        "gameCode": "XXX",
        "periode": 999990 + i,
        "logResult": "XXXX",
        "as": 9,
        "kop": 9,
        "kepala": 0,
        "ekor": 5,
        "createdDate": "2025-07-19T02:00:24.37",
        "dateResultInGame": "Jumat, 18 Jul 2025"
    }));

    // Gabungkan dummy data di awal array
    const combinedData = [...dummyData, ...dsLogPrimitif];

    await populateAnotherLogTable(combinedData, '2');
        // for (let i = 2; i < 10; i++) {
    //     await populateAnotherLogTable(data, i.toString())
    // }
}

window.generateDataLogIni = function(periodeData,awalTabel){
    let limitMaxPeriode = parseInt(periodeData) + 7;
    let limitMinPeriode = parseInt(periodeData) - 133;
    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);
    
    const filteredData = filterByPeriod(ds, limitMinPeriode, limitMaxPeriode);
    populateAnotherLogTable(filteredData, awalTabel.toString())
    //console.log(filteredData);
}

function filterByPeriod(data, startPeriod, endPeriod) {
    return data.filter(item => {
        const periode = item.periode;
        return periode >= startPeriod && periode <= endPeriod;
    }).sort((a, b) => b.periode - a.periode);
}