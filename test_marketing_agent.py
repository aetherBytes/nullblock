#!/usr/bin/env python3
"""
Test script for the NullBlock Marketing Agent
Demonstrates content generation and project analysis capabilities
"""

import requests
import json
import time

# Configuration
BASE_URL = "http://localhost:9003"
MARKETING_BASE = f"{BASE_URL}/marketing"

def test_marketing_agent():
    """Test the marketing agent endpoints"""
    
    print("🚀 Testing NullBlock Marketing Agent")
    print("=" * 50)
    
    # Test 1: Health Check
    print("\n1. 🏥 Testing Marketing Agent Health")
    try:
        response = requests.get(f"{MARKETING_BASE}/health")
        if response.status_code == 200:
            health_data = response.json()
            print(f"✅ Marketing Agent Status: {health_data.get('status', 'unknown')}")
            print(f"📊 Components: {health_data.get('components', {})}")
        else:
            print(f"❌ Health check failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Health check error: {e}")
    
    # Test 2: Get Content Themes
    print("\n2. 🎨 Testing Content Themes")
    try:
        response = requests.get(f"{MARKETING_BASE}/themes")
        if response.status_code == 200:
            themes_data = response.json()
            themes = themes_data.get('data', [])
            print(f"✅ Found {len(themes)} content themes:")
            for theme in themes:
                print(f"   - {theme.get('name', 'Unknown')}: {theme.get('description', 'No description')}")
        else:
            print(f"❌ Themes request failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Themes error: {e}")
    
    # Test 3: Generate Product Announcement Content
    print("\n3. 📝 Testing Content Generation - Product Announcement")
    try:
        content_request = {
            "content_type": "product_announcement",
            "context": {
                "topic": "New Marketing Agent",
                "audience": "developers",
                "feature": "AI-powered content generation",
                "description": "Automated Twitter content creation based on project progress"
            }
        }
        
        response = requests.post(
            f"{MARKETING_BASE}/generate-content",
            json=content_request,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            content_data = response.json()
            if content_data.get('success'):
                content = content_data.get('data', {})
                print(f"✅ Generated Content:")
                print(f"   Content: {content.get('content', 'No content')}")
                print(f"   Hashtags: {content.get('hashtags', [])}")
                print(f"   Character Count: {content.get('character_count', 0)}")
                print(f"   Engagement Score: {content.get('engagement_score', 0):.2f}")
            else:
                print(f"❌ Content generation failed: {content_data.get('error')}")
        else:
            print(f"❌ Content generation request failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Content generation error: {e}")
    
    # Test 4: Generate Technical Insight Content
    print("\n4. 🔬 Testing Content Generation - Technical Insight")
    try:
        content_request = {
            "content_type": "technical_insight",
            "context": {
                "topic": "Multi-agent orchestration",
                "audience": "technical_community",
                "insight": "How NullBlock coordinates multiple AI agents for complex workflows"
            }
        }
        
        response = requests.post(
            f"{MARKETING_BASE}/generate-content",
            json=content_request,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            content_data = response.json()
            if content_data.get('success'):
                content = content_data.get('data', {})
                print(f"✅ Generated Technical Content:")
                print(f"   Content: {content.get('content', 'No content')}")
                print(f"   Hashtags: {content.get('hashtags', [])}")
                print(f"   Character Count: {content.get('character_count', 0)}")
            else:
                print(f"❌ Technical content generation failed: {content_data.get('error')}")
        else:
            print(f"❌ Technical content request failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Technical content generation error: {e}")
    
    # Test 5: Project Analysis
    print("\n5. 🔍 Testing Project Analysis")
    try:
        response = requests.get(f"{MARKETING_BASE}/analyze-project")
        if response.status_code == 200:
            analysis_data = response.json()
            if analysis_data.get('success'):
                analysis = analysis_data.get('data', {})
                print(f"✅ Project Analysis Results:")
                print(f"   Key Opportunities: {len(analysis.get('key_opportunities', []))}")
                for i, opp in enumerate(analysis.get('key_opportunities', [])[:3], 1):
                    print(f"     {i}. {opp}")
                print(f"   Recommended Content: {len(analysis.get('recommended_content', []))}")
                for i, content in enumerate(analysis.get('recommended_content', [])[:3], 1):
                    print(f"     {i}. {content}")
                print(f"   Technical Highlights: {analysis.get('technical_highlights', [])}")
                print(f"   Target Audiences: {analysis.get('target_audiences', [])}")
            else:
                print(f"❌ Project analysis failed: {analysis_data.get('error')}")
        else:
            print(f"❌ Project analysis request failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Project analysis error: {e}")
    
    # Test 6: Create Twitter Post (Simulated)
    print("\n6. 📱 Testing Twitter Post Creation")
    try:
        post_request = {
            "content": "🚀 Just shipped our new Marketing Agent! AI-powered content generation for the NullBlock ecosystem. Building the picks and axes for the digital gold rush! #NullBlock #AgenticAI #Web3",
            "media_urls": None
        }
        
        response = requests.post(
            f"{MARKETING_BASE}/create-post",
            json=post_request,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            post_data = response.json()
            if post_data.get('success'):
                result = post_data.get('data', {})
                print(f"✅ Twitter Post Created:")
                print(f"   Success: {result.get('success')}")
                print(f"   Post ID: {result.get('post_id', 'N/A')}")
                print(f"   URL: {result.get('url', 'N/A')}")
            else:
                print(f"❌ Twitter post creation failed: {post_data.get('error')}")
        else:
            print(f"❌ Twitter post request failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Twitter post creation error: {e}")
    
    print("\n" + "=" * 50)
    print("🎉 Marketing Agent testing completed!")
    print("\n📋 Available Endpoints:")
    print(f"   GET  {MARKETING_BASE}/health")
    print(f"   GET  {MARKETING_BASE}/themes")
    print(f"   POST {MARKETING_BASE}/generate-content")
    print(f"   POST {MARKETING_BASE}/create-post")
    print(f"   GET  {MARKETING_BASE}/analyze-project")

if __name__ == "__main__":
    print("🔧 Make sure the NullBlock Agents service is running on port 9003")
    print("   Start with: cd svc/nullblock-agents && cargo run")
    print()
    
    # Wait a moment for user to start the service
    time.sleep(2)
    
    test_marketing_agent()
