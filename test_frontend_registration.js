// Test script to run in browser console
// This will test if the user registration works from the frontend

async function testUserRegistration() {
  console.log('🧪 Testing user registration from frontend...');
  
  try {
    // Import the task service
    const { taskService } = await import('./src/common/services/task-service.tsx');
    
    // Set wallet context
    const testWallet = 'test-wallet-frontend-123';
    taskService.setWalletContext(testWallet, 'solana');
    
    console.log('🔗 Task service initialized');
    console.log('📝 Testing with wallet:', testWallet);
    
    // Call registerUser
    const result = await taskService.registerUser(testWallet, 'solana');
    
    console.log('📤 Registration result:', result);
    
    if (result.success) {
      console.log('✅ User registration successful!');
      console.log('📊 User data:', result.data);
    } else {
      console.log('❌ User registration failed:', result.error);
    }
    
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

// Run the test
testUserRegistration();
