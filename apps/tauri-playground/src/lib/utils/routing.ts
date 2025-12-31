import type { SerializableNode } from '../types';

export interface Vec2 {
    x: number;
    y: number;
}

export interface Obstacle {
    min: Vec2;
    max: Vec2;
}

export function findPortPosition(nodes: SerializableNode[], portId: string): Vec2 | null {
    for (const node of nodes) {
        const inIdx = node.inputs.indexOf(portId);
        if (inIdx !== -1) {
            const count = node.inputs.length;
            const availableHeight = node.size[1] - 40; // 20px padding top/bottom
            return {
                x: node.position[0] - 1,
                y: node.position[1] + 20 + (inIdx + 0.5) * (availableHeight / count),
            };
        }
        const outIdx = node.outputs.indexOf(portId);
        if (outIdx !== -1) {
            const count = node.outputs.length;
            const availableHeight = node.size[1] - 40; // 20px padding top/bottom
            return {
                x: node.position[0] + node.size[0] + 1,
                y: node.position[1] + 20 + (outIdx + 0.5) * (availableHeight / count),
            };
        }
    }
    return null;
}

export function calculateBezierPoints(start: Vec2, end: Vec2): [Vec2, Vec2] {
    const dx = Math.abs(end.x - start.x);
    const midX = start.x + dx / 2;
    return [
        { x: midX, y: start.y },
        { x: midX, y: end.y },
    ];
}

export function calculateLinearPoints(start: Vec2, end: Vec2): Vec2[] {
    return [start, end];
}

export function getSmartOrthogonalPath(
    start: Vec2,
    end: Vec2,
    obstacles: Obstacle[],
    buffer: number = 20,
): Vec2[] {
    const outset = 20;
    const pStartReal = { x: start.x, y: start.y };
    const pStart = { x: start.x + outset, y: start.y };
    const pEndReal = { x: end.x, y: end.y };
    const pEnd = { x: end.x - outset, y: end.y };

    if (
        Math.abs(pStart.x - pEnd.x) < 1 &&
        Math.abs(pStart.y - pEnd.y) < 1
    ) {
        return [pStartReal, pEndReal];
    }

    // 1. Generate Grid
    let xs = [pStart.x, pEnd.x];
    let ys = [pStart.y, pEnd.y];

    // Global bypass lanes
    let minX = Math.min(pStart.x, pEnd.x);
    let maxX = Math.max(pStart.x, pEnd.x);
    let minY = Math.min(pStart.y, pEnd.y);
    let maxY = Math.max(pStart.y, pEnd.y);
    for (const obs of obstacles) {
        minX = Math.min(minX, obs.min.x);
        maxX = Math.max(maxX, obs.max.x);
        minY = Math.min(minY, obs.min.y);
        maxY = Math.max(maxY, obs.max.y);
    }
    xs.push(minX - 100, maxX + 100);
    ys.push(minY - 100, maxY + 100);

    for (const obs of obstacles) {
        xs.push(obs.min.x - buffer, obs.max.x + buffer);
        ys.push(obs.min.y - buffer, obs.max.y + buffer);
        xs.push(obs.min.x - buffer - 20, obs.max.x + buffer + 20);
        ys.push(obs.min.y - buffer - 20, obs.max.y + buffer + 20);
        xs.push((obs.min.x + obs.max.x) / 2);
        ys.push((obs.min.y + obs.max.y) / 2);
    }

    xs = [...new Set(xs.map((x) => Math.round(x)))].sort((a, b) => a - b);
    ys = [...new Set(ys.map((y) => Math.round(y)))].sort((a, b) => a - b);

    // 2. A* Search
    const findClosest = (val: number, arr: number[]) => {
        let minDiff = Infinity;
        let idx = 0;
        for (let i = 0; i < arr.length; i++) {
            const diff = Math.abs(arr[i] - val);
            if (diff < minDiff) {
                minDiff = diff;
                idx = i;
            }
        }
        return idx;
    };

    const startX = findClosest(pStart.x, xs);
    const startY = findClosest(pStart.y, ys);
    const endX = findClosest(pEnd.x, xs);
    const endY = findClosest(pEnd.y, ys);

    const openSet: [number, number, number, number][] = [
        [startX, startY, 0, 1],
    ]; // x, y, fScore, dir (1: Right, 2: Left, 3: Down, 4: Up)
    const gScore: Map<string, number> = new Map();
    const cameFrom: Map<string, [number, number]> = new Map();

    gScore.set(`${startX},${startY}`, 0);

    let found = false;
    while (openSet.length > 0) {
        openSet.sort((a, b) => a[2] - b[2]);
        const [cx, cy, _f, cDir] = openSet.shift()!;

        if (cx === endX && cy === endY) {
            found = true;
            break;
        }

        const neighbors = [
            [cx + 1, cy, 1], // Right
            [cx - 1, cy, 2], // Left
            [cx, cy + 1, 3], // Down
            [cx, cy - 1, 4], // Up
        ];

        for (const [nx, ny, nDir] of neighbors) {
            if (nx < 0 || nx >= xs.length || ny < 0 || ny >= ys.length)
                continue;

            // 180-degree turn check
            if (cDir !== 0) {
                if ((cDir === 1 && nDir === 2) || (cDir === 2 && nDir === 1))
                    continue;
                if ((cDir === 3 && nDir === 4) || (cDir === 4 && nDir === 3))
                    continue;
            }

            const nPos = { x: xs[nx], y: ys[ny] };
            const cPos = { x: xs[cx], y: ys[cy] };
            const mPos = {
                x: (nPos.x + cPos.x) / 2,
                y: (nPos.y + cPos.y) / 2,
            };

            let blocked = false;
            for (const obs of obstacles) {
                const isInside = (p: Vec2) =>
                    p.x >= obs.min.x &&
                    p.x <= obs.max.x &&
                    p.y >= obs.min.y &&
                    p.y <= obs.max.y;

                if (isInside(nPos) || isInside(mPos)) {
                    const distToStart = Math.abs(nPos.x - pStart.x) + Math.abs(nPos.y - pStart.y);
                    const distToEnd = Math.abs(nPos.x - pEnd.x) + Math.abs(nPos.y - pEnd.y);
                    if (distToStart < 5 || distToEnd < 5) continue;
                    blocked = true;
                    break;
                }
            }
            if (blocked) continue;

            const dist = Math.abs(nPos.x - cPos.x) + Math.abs(nPos.y - cPos.y);
            let stepCost = dist;

            // Proximity penalty
            for (const obs of obstacles) {
                if (
                    nPos.x > obs.min.x - buffer &&
                    nPos.x < obs.max.x + buffer &&
                    nPos.y > obs.min.y - buffer &&
                    nPos.y < obs.max.y + buffer
                ) {
                    stepCost += dist * 3;
                }
            }

            if (cDir !== 0 && cDir !== nDir) stepCost += 150;

            const tentativeG = gScore.get(`${cx},${cy}`)! + stepCost;

            if (tentativeG < (gScore.get(`${nx},${ny}`) ?? Infinity)) {
                cameFrom.set(`${nx},${ny}`, [cx, cy]);
                gScore.set(`${nx},${ny}`, tentativeG);
                const fScore = tentativeG + Math.abs(nPos.x - xs[endX]) + Math.abs(nPos.y - ys[endY]);
                openSet.push([nx, ny, fScore, nDir]);
            }
        }
    }

    if (found) {
        let path: Vec2[] = [];
        let curr: [number, number] | undefined = [endX, endY];
        while (curr) {
            path.push({ x: xs[curr[0]], y: ys[curr[1]] });
            curr = cameFrom.get(`${curr[0]},${curr[1]}`);
        }
        path.reverse();
        path.unshift(pStartReal);
        path.push(pEndReal);

        // Simplify path
        if (path.length > 2) {
            let simplified = [path[0]];
            for (let i = 1; i < path.length - 1; i++) {
                const pPrev = simplified[simplified.length - 1];
                const pCurr = path[i];
                const pNext = path[i + 1];

                const isColinear = (pPrev.x === pCurr.x && pCurr.x === pNext.x) || (pPrev.y === pCurr.y && pCurr.y === pNext.y);
                if (!isColinear) simplified.push(pCurr);
            }
            simplified.push(path[path.length - 1]);
            path = simplified;
        }
        return path;
    }

    return [pStartReal, pEndReal];
}
