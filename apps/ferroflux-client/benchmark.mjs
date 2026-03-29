import { performance } from 'perf_hooks';

const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

class MockBackend {
    async addNode() {
        await delay(10);
        return "node-id";
    }

    async updateNodeConfig() {
        await delay(10);
    }

    async getGraph() {
        return { nodes: {}, ports: {}, connections: [], draw_order: [] };
    }

    async getTemplates() {
        return [];
    }
}

// We will simulate the `paste` method manually
async function testPasteSequential(clipboard) {
    const backend = new MockBackend();

    for (const node of clipboard) {
        const newId = await backend.addNode();

        if (node.config) {
            for (const [k, v] of Object.entries(node.config)) {
                await backend.updateNodeConfig(newId, k, v);
            }
        }
    }
}

async function testPasteConcurrent(clipboard) {
    const backend = new MockBackend();

    for (const node of clipboard) {
        const newId = await backend.addNode();

        if (node.config) {
            await Promise.all(Object.entries(node.config).map(([k, v]) =>
                backend.updateNodeConfig(newId, k, v)
            ));
        }
    }
}

async function runBenchmark() {
    const clipboard = [
        {
            data: "template-1",
            position: { x: 0, y: 0 },
            config: {
                key1: "val1",
                key2: "val2",
                key3: "val3",
                key4: "val4",
                key5: "val5",
            }
        },
        {
            data: "template-2",
            position: { x: 0, y: 0 },
            config: {
                key1: "val1",
                key2: "val2",
                key3: "val3",
                key4: "val4",
                key5: "val5",
            }
        }
    ];

    console.log("Warming up...");
    await testPasteSequential(clipboard);
    await testPasteConcurrent(clipboard);

    console.log("Running Sequential Benchmark...");
    const startSeq = performance.now();
    await testPasteSequential(clipboard);
    const endSeq = performance.now();
    const timeSeq = endSeq - startSeq;

    console.log(`Sequential: ${timeSeq.toFixed(2)} ms`);

    console.log("Running Concurrent Benchmark...");
    const startCon = performance.now();
    await testPasteConcurrent(clipboard);
    const endCon = performance.now();
    const timeCon = endCon - startCon;

    console.log(`Concurrent: ${timeCon.toFixed(2)} ms`);

    console.log(`Improvement: ${((timeSeq - timeCon) / timeSeq * 100).toFixed(2)}%`);
}

runBenchmark();
