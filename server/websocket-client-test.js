// WebSocket client test for StelloVault
const WebSocket = require('ws');

console.log('🔌 Testing StelloVault WebSocket connection...');

// Test connection to WebSocket server
const ws = new WebSocket('ws://localhost:3001/ws');

ws.on('open', function open() {
    console.log('✅ Connected to StelloVault WebSocket server');
    
    // Send a test message
    ws.send(JSON.stringify({
        type: 'TEST_CONNECTION',
        message: 'Hello from test client',
        timestamp: new Date().toISOString()
    }));
    
    console.log('📨 Sent test message');
});

ws.on('message', function message(data) {
    try {
        const parsed = JSON.parse(data);
        console.log('📬 Received message:', parsed);
        
        if (parsed.type === 'CONNECTION_ESTABLISHED') {
            console.log('🎉 WebSocket connection test successful!');
            
            // Test expected event formats
            console.log('\n📋 Expected event formats:');
            console.log('ESCROW_CREATED:', { type: "ESCROW_CREATED", escrowId: "uuid", buyerId: "uuid", sellerId: "uuid" });
            console.log('ESCROW_UPDATED:', { type: "ESCROW_UPDATED", escrowId: "uuid", status: "ACTIVE" });
            console.log('LOAN_UPDATED:', { type: "LOAN_UPDATED", loanId: "uuid", status: "REPAID" });
            console.log('GOVERNANCE_VOTE_CAST:', { type: "GOVERNANCE_VOTE_CAST", proposalId: "uuid", newTally: 42 });
            
            setTimeout(() => {
                ws.close();
                console.log('\n✨ Test completed');
            }, 1000);
        }
    } catch (error) {
        console.log('📬 Received raw message:', data.toString());
    }
});

ws.on('error', function error(err) {
    console.error('❌ WebSocket error:', err.message);
    if (err.message.includes('ECONNREFUSED')) {
        console.log('\n💡 Make sure the StelloVault server is running on port 3001');
        console.log('   Run: npm run dev');
    }
});

ws.on('close', function close() {
    console.log('🔌 WebSocket connection closed');
});

// Timeout after 10 seconds
setTimeout(() => {
    if (ws.readyState === WebSocket.CONNECTING) {
        console.error('❌ Connection timeout - server may not be running');
        ws.terminate();
    }
}, 10000);
