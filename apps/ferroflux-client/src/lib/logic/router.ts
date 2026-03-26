import type { Vec2 } from "$lib/types";

export interface Rect {
    x: number;
    y: number;
    w: number;
    h: number;
}

// Configuration
const GRID_MARGIN = 20; // Distance from obstacle to route line

function snap(v: number): number {
    return Math.round(v);
}

export function findRoute(startRaw: Vec2, endRaw: Vec2, obstaclesRaw: Rect[]): Vec2[] {
    const start = { x: snap(startRaw.x), y: snap(startRaw.y) };
    const end = { x: snap(endRaw.x), y: snap(endRaw.y) };
    const obstacles = obstaclesRaw.map(o => ({
        x: snap(o.x),
        y: snap(o.y),
        w: snap(o.w),
        h: snap(o.h)
    }));

    // 2. Build the Hanan Grid
    // Collect all interesting X and Y coordinates
    const xPoints = new Set<number>();
    const yPoints = new Set<number>();

    // Start/End
    const xCoords = new Set<number>([start.x, end.x]);
    const yCoords = new Set<number>([start.y, end.y]);

    obstacles.forEach(obs => {
        // Expand obstacle by margin for routing lines
        const left = obs.x - GRID_MARGIN;
        const right = obs.x + obs.w + GRID_MARGIN;
        const top = obs.y - GRID_MARGIN;
        const bottom = obs.y + obs.h + GRID_MARGIN;

        xCoords.add(left);
        xCoords.add(right);
        yCoords.add(top);
        yCoords.add(bottom);
    });

    // Sort coords
    const sortedX = Array.from(xCoords).sort((a, b) => a - b);
    const sortedY = Array.from(yCoords).sort((a, b) => a - b);

    // Add Midpoints to allow routing through empty space
    const xWithMids = [...sortedX];
    const yWithMids = [...sortedY];

    // Helper to inject mids
    const injectMids = (source: number[], target: number[]) => {
        for (let i = 0; i < source.length - 1; i++) {
            const gap = source[i + 1] - source[i];
            if (gap > 40) { // If gap is large enough
                target.push(Math.round((source[i] + source[i + 1]) / 2));
            }
        }
    };
    injectMids(sortedX, xWithMids);
    injectMids(sortedY, yWithMids);

    // Sort again
    xWithMids.sort((a, b) => a - b);
    yWithMids.sort((a, b) => a - b);

    // Use unique
    const finalX = [...new Set(xWithMids)];
    const finalY = [...new Set(yWithMids)];

    // 3. A* Search
    const nodes = new Map<string, { x: number, y: number, g: number, h: number, f: number, parent: string | null }>();
    const openSet = new Set<string>();
    const closedSet = new Set<string>();

    const startKey = `${start.x},${start.y}`;
    const endKey = `${end.x},${end.y}`;

    // Helper: Is Valid Segment?
    // We check if the line segment from p1 to p2 intersects any obstacle (expanded by smaller margin e.g. 1px to allow grazing)
    // Actually we essentially check if the midpoint of the grid cell is inside obstacle?
    // No, we are moving along the grid lines. We check if the segment is effectively "inside" an obstacle.

    // Simplification: Check if the *segment* overlaps strictly with any original obstacle Rect (slightly shrunk?)
    // Or rather: Is the segment valid?
    // A segment is invalid if it passes *through* an obstacle.
    // Grazing (touching border) is usually allowed in Hanan, but we inflated the grid lines by Margin.
    // So the grid lines themselves are "safe" unless they are *inside* a node?
    // Grid lines are generated *around* nodes.
    // But a line from Node A top to Node B top might cross Node C.
    // So we must check collision.

    // ... Check collision ... (uses original obstacles)
    function isSegmentBlocked(p1: Vec2, p2: Vec2): boolean {
        // Simple AABB overlap check for segment
        const segMinX = Math.min(p1.x, p2.x);
        const segMaxX = Math.max(p1.x, p2.x);
        const segMinY = Math.min(p1.y, p2.y);
        const segMaxY = Math.max(p1.y, p2.y);

        // Treat segment as rectangle
        for (const obs of obstacles) {
            // Shrink obstacle slightly to allow grazing? Or Strict?
            // Obstacles are the nodes.
            const obsLeft = obs.x;
            const obsRight = obs.x + obs.w;
            const obsTop = obs.y;
            const obsBottom = obs.y + obs.h;

            // Check Rect Intersection (Strict inequality means touching is OK)
            if (segMinX < obsRight && segMaxX > obsLeft && segMinY < obsBottom && segMaxY > obsTop) {
                return true; // Blocked
            }
        }
        return false;
    }

    nodes.set(startKey, { x: start.x, y: start.y, g: 0, h: manhattan(start, end), f: manhattan(start, end), parent: null });
    openSet.add(startKey);

    while (openSet.size > 0) {
        // Get lowest F
        // ... (Find lowest F) ...
        let currentKey = "";
        let lowestF = Infinity;
        for (const key of openSet) {
            const n = nodes.get(key)!;
            if (n.f < lowestF) {
                lowestF = n.f;
                currentKey = key;
            }
        }

        if (currentKey === "") break; // Should not happen
        if (currentKey === endKey) break; // Found

        openSet.delete(currentKey);
        closedSet.add(currentKey);

        const current = nodes.get(currentKey)!;

        // Find Neighbors in Hanan Grid
        // A point in Hanan grid is (sortedX[i], sortedY[j])
        // We look for index in sortedX and sortedY? 
        // Or just iterate all points? No, too slow.
        // We just move UP, DOWN, LEFT, RIGHT to the *next* grid line.

        // Find neighbors in Final Grid
        const xi = finalX.indexOf(current.x);
        const yi = finalY.indexOf(current.y);

        // Potential neighbors: (x[xi-1], y[yi]), (x[xi+1], y[yi]), ...
        const neighborCoords: Vec2[] = [];
        if (xi > 0) neighborCoords.push({ x: finalX[xi - 1], y: finalY[yi] });
        if (xi < finalX.length - 1) neighborCoords.push({ x: finalX[xi + 1], y: finalY[yi] });
        if (yi > 0) neighborCoords.push({ x: finalX[xi], y: finalY[yi - 1] });
        if (yi < finalY.length - 1) neighborCoords.push({ x: finalX[xi], y: finalY[yi + 1] });

        // Also we must include End point if it's not exactly on grid? 
        // We added start/end to sorted sets, so they ARE on grid.

        for (const neighbor of neighborCoords) {
            const neighborKey = `${neighbor.x},${neighbor.y}`;
            if (closedSet.has(neighborKey)) continue;

            // Check Collision
            if (isSegmentBlocked({ x: current.x, y: current.y }, neighbor)) continue;

            // G Cost
            const dist = manhattan({ x: current.x, y: current.y }, neighbor);

            // Turn Penalty
            let turnPenalty = 0;
            if (current.parent) {
                const parent = nodes.get(current.parent)!;
                const prevDx = current.x - parent.x;
                const prevDy = current.y - parent.y;
                const newDx = neighbor.x - current.x;
                const newDy = neighbor.y - current.y;

                // If direction changed
                if ((prevDx !== 0 && newDy !== 0) || (prevDy !== 0 && newDx !== 0)) {
                    turnPenalty = 20;
                }
            }

            // Central Bias (Prefer routing through the geometric center of the connection)
            const midX = (start.x + end.x) / 2;
            const midY = (start.y + end.y) / 2;

            let centerBias = 0;
            // If Vertical Move (changing Y), we want to be near midX
            if (neighbor.x === current.x && neighbor.y !== current.y) {
                centerBias = Math.abs(neighbor.x - midX) * 0.1;
            }
            // If Horizontal Move (changing X), we want to be near midY
            else if (neighbor.y === current.y && neighbor.x !== current.x) {
                centerBias = Math.abs(neighbor.y - midY) * 0.1;
            }

            const tentativeG = current.g + dist + turnPenalty + centerBias;

            let neighborNode = nodes.get(neighborKey);
            if (!neighborNode) {
                neighborNode = { x: neighbor.x, y: neighbor.y, g: Infinity, h: manhattan(neighbor, end), f: Infinity, parent: null };
                nodes.set(neighborKey, neighborNode);
            }

            if (tentativeG < neighborNode.g) {
                neighborNode.parent = currentKey;
                neighborNode.g = tentativeG;
                neighborNode.f = neighborNode.g + neighborNode.h;
                if (!openSet.has(neighborKey)) openSet.add(neighborKey);
            }
        }
    }

    // Reconstruct Path
    const path: Vec2[] = [];
    let curr = nodes.get(endKey);
    if (!curr) {
        // Fallback: Direct line if A* failed (e.g. completely enclosed?)
        return [start, end];
    }

    while (curr) {
        path.unshift({ x: curr.x, y: curr.y });
        if (curr.parent) curr = nodes.get(curr.parent);
        else curr = undefined;
    }

    // Simplify Path (Remove collinear points)
    if (path.length < 3) return path;

    const simplified: Vec2[] = [path[0]];
    for (let i = 1; i < path.length - 1; i++) {
        const prev = simplified[simplified.length - 1];
        const curr = path[i];
        const next = path[i + 1];

        // Check if collinear
        const dx1 = curr.x - prev.x;
        const dy1 = curr.y - prev.y;
        const dx2 = next.x - curr.x;
        const dy2 = next.y - curr.y;

        // Normalize direction (simple check for 90 deg grid)
        const isHorizontal1 = dy1 === 0;
        const isHorizontal2 = dy2 === 0;

        if (isHorizontal1 === isHorizontal2) {
            // Same direction, skip current
            continue;
        }
        simplified.push(curr);
    }
    simplified.push(path[path.length - 1]);

    return simplified;
}

function manhattan(a: Vec2, b: Vec2): number {
    return Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
}
