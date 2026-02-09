/**
 * TradeX Analysis Utilities
 * Pattern matching and analysis functions for log data
 */

/**
 * Extract digits from formatted result string or log object
 * @param {string|object} input - e.g. "1.234" or "1234" or log object
 * @returns {object|null} - { as, kop, kepala, ekor } or null
 */
export function getDigits(input) {
    if (!input) return null;
    
    // Handle log object
    let formattedResult;
    if (typeof input === 'object') {
        formattedResult = input.formatted_result || input.logResult;
    } else {
        formattedResult = input;
    }
    
    if (!formattedResult || typeof formattedResult !== 'string') return null;
    const raw = formattedResult.replace('.', '').trim();
    if (raw.length < 4) return null;
    return {
        as: parseInt(raw[0]),
        kop: parseInt(raw[1]),
        kepala: parseInt(raw[2]),
        ekor: parseInt(raw[3])
    };
}

/**
 * Get 3-digit front (AS + KOP + KEPALA)
 */
export function getFront3(log) {
    const d = getDigits(log.formatted_result || log.logResult);
    if (!d) return null;
    return `${d.as}${d.kop}${d.kepala}`;
}

/**
 * Get 3-digit back (KOP + KEPALA + EKOR)
 */
export function getBack3(log) {
    const d = getDigits(log.formatted_result || log.logResult);
    if (!d) return null;
    return `${d.kop}${d.kepala}${d.ekor}`;
}

/**
 * Find logs with same 3-digit front pattern
 * @param {array} logs - All log data
 * @param {object} selectedLog - The log to match against
 * @returns {array} - Matching logs sorted by periode descending
 */
export function findFront3Matches(logs, selectedLog) {
    const selFront3 = getFront3(selectedLog);
    if (!selFront3) return [];
    
    const selPeriode = selectedLog.periode || selectedLog.Periode;
    
    return logs
        .filter(log => {
            const logPeriode = log.periode || log.Periode;
            if (logPeriode > selPeriode) return false; // Only current + past
            return getFront3(log) === selFront3;
        })
        .sort((a, b) => (b.periode || b.Periode) - (a.periode || a.Periode));
}

/**
 * Find logs with same 3-digit back pattern
 * @param {array} logs - All log data
 * @param {object} selectedLog - The log to match against
 * @returns {array} - Matching logs sorted by periode descending
 */
export function findBack3Matches(logs, selectedLog) {
    const selBack3 = getBack3(selectedLog);
    if (!selBack3) return [];
    
    const selPeriode = selectedLog.periode || selectedLog.Periode;
    
    return logs
        .filter(log => {
            const logPeriode = log.periode || log.Periode;
            if (logPeriode > selPeriode) return false; // Only current + past
            return getBack3(log) === selBack3;
        })
        .sort((a, b) => (b.periode || b.Periode) - (a.periode || a.Periode));
}

/**
 * Build P(rev), C(urrent), N(ext) list for matched logs
 * @param {array} matchedLogs - Logs that matched the pattern
 * @param {array} allLogs - All available logs
 * @param {number} limit - Max number of results (default 15)
 * @returns {array} - Array of { prev, current, next, periode, prevTrend, currentTrend, nextTrend }
 */
export function buildPCNList(matchedLogs, allLogs, limit = 15) {
    return matchedLogs.slice(0, limit).map(log => {
        const logPeriode = log.periode || log.Periode;
        const prevPeriode = logPeriode - 1;
        const nextPeriode = logPeriode + 1;

        const prevLog = allLogs.find(l => (l.periode || l.Periode) == prevPeriode);
        const nextLog = allLogs.find(l => (l.periode || l.Periode) == nextPeriode);

        return {
            prev: prevLog ? (prevLog.formatted_result || prevLog.logResult || '----') : '----',
            prevTrend: prevLog ? (prevLog.trend || '') : '',
            current: log.formatted_result || log.logResult || '----',
            currentTrend: log.trend || '',
            next: nextLog ? (nextLog.formatted_result || nextLog.logResult || '----') : '----',
            nextTrend: nextLog ? (nextLog.trend || '') : '',
            periode: logPeriode
        };
    });
}

/**
 * Check if two logs match on pattern (for cell highlighting)
 * @param {object} curr - Current log
 * @param {object} compare - Compare log
 * @returns {string} - CSS border style or empty string
 */
export function checkPatternMatch(curr, compare) {
    if (!curr || !compare) return "";
    
    const c = getDigits(curr.formatted_result || curr.logResult);
    const o = getDigits(compare.formatted_result || compare.logResult);
    if (!c || !o) return "";

    // Front match (AS+KOP)
    if (c.as == o.as && c.kop == o.kop) return "border:2px solid orangered";
    if (c.as == o.kop && c.kop == o.kepala) return "border-bottom:2px solid orangered";
    if (c.as == o.kepala && c.kop == o.ekor) return "border-bottom:2px solid orangered";
    
    // Mid match (KOP+KEPALA)
    if (c.kop == o.as && c.kepala == o.kop) return "border-bottom:2px solid yellow";
    if (c.kop == o.kop && c.kepala == o.kepala) return "border:2px solid yellow";
    if (c.kop == o.kepala && c.kepala == o.ekor) return "border-bottom:2px solid yellow";

    // Back match (KEPALA+EKOR)
    if (c.kepala == o.as && c.ekor == o.kop) return "border-bottom:2px solid green";
    if (c.kepala == o.kop && c.ekor == o.kepala) return "border-bottom:2px solid green";
    if (c.kepala == o.kepala && c.ekor == o.ekor) return "border:2px solid green";

    return "";
}
