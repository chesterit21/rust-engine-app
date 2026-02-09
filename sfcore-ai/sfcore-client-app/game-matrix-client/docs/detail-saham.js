window.populateChartResult = function (data) {
    let divChart = $('#divChartLog');
    divChart.empty();
    divChart.append('<canvas id="chartLog" style="width: 1800px;height: 600px;"></canvas>');

    let codes = [];
    let dataResults = [];
    let backgrounds = [];
    let borders = [];
    let currentLog;

    var gameCode = $('#hidCode').val();

    let countRed = 0;
    let countYellow = 0;
    let countGreen = 0;
    let countFill = false;
    for (let i = 0; i < data.length; i++) {
        if (countFill == false) {
            const log = data[i];
            if (log.gameCode != 'HK' || log.gameCode != 'SD' || log.gameCode != 'SGP') {
                if (i == 0) {
                    let codeTemplate = log.periode + '*CX';
                    if (gameCode == 'TMD') codeTemplate = log.periode + '*BX';

                    codes.push(codeTemplate);
                    dataResults.push(log.logResult);
                    let bgRand = backgroundCollection();
                    backgrounds.push(bgRand[0].bg);
                    borders.push(bgRand[0].border);
                    currentLog = data[0];
                }
                else {

                    let codeTemplate = '';
                    let borderSameData = checkDataLogResultIfSameData(currentLog, log);
                    if (borderSameData == 'orangered') {
                        countRed++;
                        codeTemplate = log.periode + '*CX';
                    }
                    else if (borderSameData == 'yellow') {
                        countYellow++;
                        codeTemplate = log.periode + '*CX';
                    }
                    else if (borderSameData == 'green') {
                        countGreen++;
                        codeTemplate = log.periode + '*CX';
                    }
                    else {
                        codeTemplate = log.periode + '*XB';
                    }
                    codes.push(codeTemplate);
                    dataResults.push(log.logResult);

                    backgrounds.push(borderSameData);
                    borders.push(borderSameData);
                    if (countRed >= 2) {
                        if (countYellow >= 2) {
                            if (countRed >= 4 && countYellow >= 4) {
                                countFill = true;
                            }
                            if (countGreen >= 2) {
                                countFill = true;
                            }
                        }
                    }
                }


            }
        }
        else {
            break;
        }
    }


    const ctx = document.getElementById('chartLog');
    const myChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: codes,
            datasets: [{
                label: '#INDEX RESULT ' + gameCode,
                data: dataResults,
                backgroundColor: backgrounds,
                borderColor: borders,
                borderWidth: 1
            }]
        },
        options: {
            elements: {
                point: {
                    hoverRadius: 10,
                    hoverBorderWidth: 2,
                    hoverBorderColor: '#fff'
                },
                line: {
                    hoverBorderWidth: 2
                }
            },
            onHover: (event, chartElement) => {
                event.native.target.style.cursor = chartElement[0] ? 'pointer' : 'default';
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            onClick: (event, elements) => {
                if (elements.length > 0) {
                    const clickedElementIndex = elements[0].index;
                    const datasetIndex = elements[0].datasetIndex;

                    //console.log(myChart.data.datasets[datasetIndex]);
                    //console.log(myChart.data.labels[clickedElementIndex]);
                    let peridoe = myChart.data.labels[clickedElementIndex].split('*');
                    //console.log(peridoe);
                    changeBackground(peridoe[0], undefined, peridoe[1]);
                }
            }
        }
    });
    myChart.update('active');

    const ctxLine = document.getElementById('chartLogLine');
    const myChartLine = new Chart(ctxLine, {
        type: 'bar',
        data: {
            labels: codes,
            datasets: [{
                label: '#SAHAM HARIAN ' + gameCode,
                data: dataResults,
                backgroundColor: backgrounds,
                borderColor: borders,
                borderWidth: 1
            }]
        },
        options: {
            onHover: (event, chartElement) => {
                event.native.target.style.cursor = chartElement[0] ? 'pointer' : 'default';
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            onClick: (event, elements) => {
                if (elements.length > 0) {
                    const clickedElementIndex = elements[0].index;
                    const datasetIndex = elements[0].datasetIndex;

                    //console.log(myChart.data.datasets[datasetIndex]);
                    //let peridoe = myChartLine.data.labels[clickedElementIndex];
                    let peridoe = myChartLine.data.labels[clickedElementIndex].split('*');
                    console.log(myChartLine.data.labels[clickedElementIndex]);
                    //changeBackground(peridoe[0],undefined,'XB');
                    changeBackground(peridoe[0], undefined, peridoe[1]);

                }
            }
        }
    });
    myChartLine.update('active');

}

function backgroundCollection() {
    const array = [
        'rgba(250, 28, 76, 0.48)',
        'rgba(145, 235, 54, 0.84)',
        'rgba(163, 68, 204, 0.41)',
        'rgba(246, 234, 62, 0.84)'
    ];
    const randomIndex = Math.floor(Math.random() * array.length);
    const randomValue = array[randomIndex];
    let backgroundAndBorders = [];
    if (randomValue == 'rgba(250, 28, 76, 0.48)') { backgroundAndBorders.push({ bg: randomValue, border: 'rgb(255, 99, 132)' }); }
    else if (randomValue == 'rgba(255, 118, 154, 0.54)') { backgroundAndBorders.push({ bg: randomValue, border: 'rgba(198, 73, 212, 0.63)' }); }
    else if (randomValue == 'rgba(145, 235, 54, 0.84)') { backgroundAndBorders.push({ bg: randomValue, border: 'rgb(99, 255, 211)' }); }
    else if (randomValue == 'rgba(163, 68, 204, 0.41)') { backgroundAndBorders.push({ bg: randomValue, border: 'rgb(226, 99, 255)' }); }
    else if (randomValue == 'rgba(36, 205, 179, 0.52)') { backgroundAndBorders.push({ bg: randomValue, border: 'rgb(52, 156, 99)' }); }
    else {
        backgroundAndBorders.push({ bg: randomValue, border: 'rgba(255, 255, 255, 0.78)' });
    }
    return backgroundAndBorders;
}


window.checkDataLogResultIfSameData = function (currentLog, compareLog) {


    if (currentLog.as == compareLog.as && currentLog.kop == compareLog.kop) {
        return "orangered";
    }
    else if (currentLog.as == compareLog.kop && currentLog.kop == compareLog.kepala) {
        return "orangered";
    }
    else if (currentLog.as == compareLog.kepala && currentLog.kop == compareLog.ekor) {
        return "orangered";
    }
    else if (currentLog.kop == compareLog.as && currentLog.kepala == compareLog.kop) {
        return "yellow";
    }
    else if (currentLog.kop == compareLog.kop && currentLog.kepala == compareLog.kepala) {
        return "yellow";
    }
    else if (currentLog.kop == compareLog.kepala && currentLog.kepala == compareLog.ekor) {
        return "yellow";
    }
    else if (currentLog.kepala == compareLog.as && currentLog.ekor == compareLog.kop) {
        return "green";
    }
    else if (currentLog.kepala == compareLog.kop && currentLog.ekor == compareLog.kepala) {
        return "green";
    }
    else if (currentLog.kepala == compareLog.kepala && currentLog.ekor == compareLog.ekor) {
        return "green";
    }
    else {
        return "gray";
    }
}

window.populateChartLogFront = function (data, coloring) {
    let div = $('.dv-chart-f');
    div.empty();
    div.append('<canvas id="chartDataFront" width="1200px" style="width:100%;height: 200px;"></canvas>');

    let codes = [];
    let dataResults = [];
    let backgrounds = [];
    let borders = [];

    sessionStorage.removeItem('logSelectedFront');
    let logSelected = [];
    for (let i = 0; i < data.length; i++) {
        const log = data[i];
        let codeTemplate = log.periode + '#' + log.logResult;
        codes.push(codeTemplate);
        dataResults.push(log.logResult);
        if (i == 10) {
            backgrounds.push('rgba(227, 15, 185, 0.76)');
            borders.push('rgba(177, 51, 255, 0.99)');
            logSelected.push({ logResult: log.logResult, color: coloring });
        } else {
            backgrounds.push('rgba(222, 30, 120, 0.76)');
            borders.push('rgba(226, 20, 182, 0.5)');
        }
    }

    const ctx = document.getElementById('chartDataFront');
    const myChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: codes,
            datasets: [{
                label: '#DATA FRONT',
                data: dataResults,
                backgroundColor: backgrounds,
                borderColor: borders,
                borderWidth: 2
            }]
        },
        options: {
            elements: {
                point: {
                    hoverRadius: 15,
                    hoverBorderWidth: 3,
                    hoverBorderColor: '#fff'
                },
                line: {
                    hoverBorderWidth: 3
                }
            },
            onHover: (event, chartElement) => {
                event.native.target.style.cursor = chartElement[0] ? 'pointer' : 'default';
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            onClick: (event, elements) => {
                if (elements.length > 0) {
                    const clickedElementIndex = elements[0].index;
                    const datasetIndex = elements[0].datasetIndex;

                    // console.log(myChart.data.datasets[datasetIndex]);
                    // console.log(myChart.data.labels[clickedElementIndex]);
                    let dataPeriodoe = myChart.data.labels[clickedElementIndex].split('#');
                    changeBgSelectedData(dataPeriodoe[0], dataPeriodoe[1]);
                    // autoSaveDataLogInChildChart(dataPeriodoe[1]);
                }
            }
        }
    });
    myChart.update('active');
    sessionStorage.setItem('logSelectedFront', JSON.stringify(logSelected));

}

window.populateChartLogMid = function (data, coloring) {
    let div = $('.dv-chart-m');
    div.empty();
    div.append('<canvas id="chartDataMid"  width="1200px" style="width:100%;height: 200px;"></canvas>');

    let codes = [];
    let dataResults = [];
    let backgrounds = [];
    let borders = [];
    sessionStorage.removeItem('logSelectedMid');
    let logSelected = [];

    for (let i = 0; i < data.length; i++) {
        const log = data[i];
        let codeTemplate = log.periode + '#' + log.logResult;
        codes.push(codeTemplate);
        dataResults.push(log.logResult);
        if (i == 10) {
            backgrounds.push('rgba(227, 15, 185, 0.76)');
            borders.push('rgba(177, 51, 255, 0.99)');
            logSelected.push({ logResult: log.logResult, color: coloring });
        } else {
            backgrounds.push('rgba(222, 222, 30, 0.76)');
            borders.push('rgba(255, 255, 177, 0.5)');
        }
    }


    const ctx = document.getElementById('chartDataMid');
    const myChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: codes,
            datasets: [{
                label: '#DATA MID',
                data: dataResults,
                backgroundColor: backgrounds,
                borderColor: borders,
                borderWidth: 2
            }]
        },
        options: {
            elements: {
                point: {
                    hoverRadius: 15,
                    hoverBorderWidth: 3,
                    hoverBorderColor: '#fff'
                },
                line: {
                    hoverBorderWidth: 3
                }
            },
            onHover: (event, chartElement) => {
                event.native.target.style.cursor = chartElement[0] ? 'pointer' : 'default';
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            onClick: (event, elements) => {
                if (elements.length > 0) {
                    const clickedElementIndex = elements[0].index;
                    const datasetIndex = elements[0].datasetIndex;

                    // console.log(myChart.data.datasets[datasetIndex]);
                    // console.log(myChart.data.labels[clickedElementIndex]);
                    let dataPeriodoe = myChart.data.labels[clickedElementIndex].split('#');
                    changeBgSelectedData(dataPeriodoe[0], dataPeriodoe[1]);
                    // autoSaveDataLogInChildChart(dataPeriodoe[1]);
                }
            }
        }
    });
    myChart.update('active');
    sessionStorage.setItem('logSelectedMid', JSON.stringify(logSelected));

}

window.populateChartLogBack = function (data, coloring) {
    let div = $('.dv-chart-b');
    div.empty();
    div.append('<canvas id="chartDataBack"  width="1200px" style="width:100%;height: 200px;"></canvas>');

    let codes = [];
    let dataResults = [];
    let backgrounds = [];
    let borders = [];

    sessionStorage.removeItem('logSelectedBack');
    let logSelected = [];
    for (let i = 0; i < data.length; i++) {
        const log = data[i];
        let codeTemplate = log.periode + '#' + log.logResult;
        codes.push(codeTemplate);
        dataResults.push(log.logResult);
        if (i == 10) {
            backgrounds.push('rgba(227, 15, 185, 0.76)');
            borders.push('rgba(177, 51, 255, 0.99)');
            logSelected.push({ logResult: log.logResult, color: coloring });
        } else {
            backgrounds.push('rgba(10, 134, 47, 0.76)');
            borders.push('rgba(21, 235, 10, 0.72)');
        }
    }


    const ctx = document.getElementById('chartDataBack');
    const myChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: codes,
            datasets: [{
                label: '#DATA BACK',
                data: dataResults,
                backgroundColor: backgrounds,
                borderColor: borders,
                borderWidth: 2
            }]
        },
        options: {
            elements: {
                point: {
                    hoverRadius: 15,
                    hoverBorderWidth: 3,
                    hoverBorderColor: '#fff'
                },
                line: {
                    hoverBorderWidth: 3
                }
            },
            onHover: (event, chartElement) => {
                event.native.target.style.cursor = chartElement[0] ? 'pointer' : 'default';
            },
            scales: {
                y: {
                    beginAtZero: true
                }
            },
            onClick: (event, elements) => {
                if (elements.length > 0) {
                    const clickedElementIndex = elements[0].index;
                    const datasetIndex = elements[0].datasetIndex;

                    // console.log(myChart.data.datasets[datasetIndex]);
                    // console.log(myChart.data.labels[clickedElementIndex]);
                    let dataPeriodoe = myChart.data.labels[clickedElementIndex].split('#');
                    changeBgSelectedData(dataPeriodoe[0], dataPeriodoe[1]);
                    // autoSaveDataLogInChildChart(dataPeriodoe[1]);
                }
            }
        }
    });
    myChart.update('active');
    sessionStorage.setItem('logSelectedBack', JSON.stringify(logSelected));

}

async function generateTableLogSummary() {
    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);
    await populateDataLogSummary(ds);
}

async function simulationDataIncrement() {
    let ds = sessionStorage.getItem('dsLogGame');
    ds = JSON.parse(ds);
    let nom = $('#tbnom').val();
    let ddlContNum = $('.ddl-count-num').val();
    let currentLog = [];
    let collectPeriode = [];

    $('.global-td-inc').each(function () {
        $(this).css('background-color', '#1f212e');
        $(this).css('color', '#909090ff');
    });

    const positionConfig = {
        '1': { logProps: ['as', 'kop'], currentProps: ['as', 'kop'], color: 'crimson', textColor: '#949494ff' },
        '2': { logProps: ['kop', 'kepala'], currentProps: ['kop', 'kepala'], color: 'green', textColor: '#949494ff' },
        '3': { logProps: ['kepala', 'ekor'], currentProps: ['kepala', 'ekor'], color: '#d9cd20ff', textColor: '#949494ff' },
        '4': { logProps: ['as', 'kepala'], currentProps: ['as', 'kepala'], color: '#6350b4ff', textColor: '#949494ff' },
        '5': { logProps: ['as', 'ekor'], currentProps: ['as', 'ekor'], color: '#7f1865ff', textColor: '#949494ff' },
        '6': { logProps: ['kop', 'ekor'], currentProps: ['kop', 'ekor'], color: '#396c5eff', textColor: '#949494ff' }
    };

    for (var a = 0; a < ds.length; a++) {
        var log = ds[a];
        if (a > 0) {
            const config = positionConfig[ddlContNum];
            if (!config) continue;

            const [prop1, prop2] = config.logProps;
            const aa = parseInt(log[prop1]) + parseInt(nom);
            const bb = parseInt(log[prop2]) + parseInt(nom);

            const n1 = (aa % 10).toString();
            const n2 = (bb % 10).toString();

            const [currProp1, currProp2] = config.currentProps;
            if (currentLog[0][currProp1].toString() === n1 && currentLog[0][currProp2].toString() === n2) {
                $(`.tdp-${log.periode}`).css('background-color', config.color).css('color', config.textColor);
                collectPeriode.push(log.periode);
            }
        }
        currentLog = [];
        currentLog.push(log);
    }

    let divBox = $('#row-long-inc');
    divBox.empty();
    let topPeriode = 0;
    for (let b = 0; b < collectPeriode.length; b++) {
        let periode = collectPeriode[b];
        if (topPeriode != 0) {
            let gap = parseInt(topPeriode) - parseInt(periode);
            divBox.append('<div class="col-lg-1" style="background-color: #303030ff;color: #7cc544ff;border: 1px solid gray;border-radius: 5px;padding: 5px;margin: 2px;">' + gap + '</div>');
        }
        topPeriode = periode;
    }

    ds.slice(0, 1);
    let currentDataLog = ds[0];
    $('#row-suggest-inc').empty();
    const configSelect = positionConfig[ddlContNum];
    if (!configSelect) return;

    const [propOne, propTwo] = configSelect.logProps;
    const aaa = parseInt(currentDataLog[propOne]) + parseInt(nom);
    const bbb = parseInt(currentDataLog[propTwo]) + parseInt(nom);

    const nOne = (aaa % 10).toString();
    const nTwo = (bbb % 10).toString();
    const finalNom = nOne + nTwo;
    $('#row-suggest-inc').append('<div class="col-lg-4" style="background-color: #303030ff;color: #7cc544ff;border: 1px solid gray;border-radius: 5px;padding: 5px;margin: 2px;font-size: 18px; font-weight: bold">'+currentDataLog.logResult+' ==> '+finalNom+' </div>');

}

async function populateDataLogSummary(data) {

    var tbl = $('.tblList-log-summary>tbody');
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
                //let borderColor = checkLogIfSameData(currentLog, dt[i]);
                const isSpecialCell = (baris - i === 3) || (baris - i === 8);
                let style = 'cursor:pointer;background-color : #1f212e;';
                row += `<td title="${dt[i].log}" style="${style}" class="tdp-${dt[i].periode} global-td-inc" onclick='changeBackground("${dt[i].periode}",null,null,"V")'>x-xx-x</td>`;
                if (baris === 9 && i === 6) {
                    baris -= 5;
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
    }
}
