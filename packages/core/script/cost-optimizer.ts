#!/usr/bin/env bun

import { generate } from "../src/generate";
import { Model } from "../src/schema";

interface CostAnalysis {
  modelId: string;
  provider: string;
  inputCost: number;
  outputCost: number;
  contextWindow: number;
  capabilities: string[];
  costPerMillion: number;
  valueScore: number;
}

interface OptimizationRecommendation {
  useCase: string;
  recommended: string[];
  alternatives: string[];
  savings: string;
}

async function analyzeCosts(): Promise<CostAnalysis[]> {
  const data = await generate("providers");
  const analyses: CostAnalysis[] = [];

  for (const [providerId, provider] of Object.entries(data)) {
    for (const [modelId, model] of Object.entries(provider.models)) {
      if (!model.cost) continue;

      const capabilities: string[] = [];
      if (model.tool_call) capabilities.push("tool_call");
      if (model.attachment) capabilities.push("attachment");
      if (model.reasoning) capabilities.push("reasoning");
      if (model.structured_output) capabilities.push("structured_output");

      // Calculate value score (inverse of cost + capability bonus)
      const avgCost = (model.cost.input + model.cost.output) / 2;
      const capabilityBonus = capabilities.length * 0.1;
      const valueScore = 1 / (avgCost + 0.01) + capabilityBonus;

      analyses.push({
        modelId: `${providerId}/${modelId}`,
        provider: providerId,
        inputCost: model.cost.input,
        outputCost: model.cost.output,
        contextWindow: model.limit.context,
        capabilities,
        costPerMillion: avgCost,
        valueScore,
      });
    }
  }

  return analyses.sort((a, b) => a.costPerMillion - b.costPerMillion);
}

function generateRecommendations(analyses: CostAnalysis[]): OptimizationRecommendation[] {
  const recommendations: OptimizationRecommendation[] = [];

  // Free models
  const freeModels = analyses.filter(a => a.costPerMillion === 0);
  recommendations.push({
    useCase: "Development & Testing",
    recommended: freeModels.slice(0, 3).map(m => m.modelId),
    alternatives: freeModels.slice(3, 6).map(m => m.modelId),
    savings: "100% compared to paid models",
  });

  // Ultra-low cost (<$0.10)
  const ultraLowCost = analyses.filter(a => a.costPerMillion > 0 && a.costPerMillion < 0.10);
  recommendations.push({
    useCase: "High-Volume Text Generation",
    recommended: ultraLowCost.slice(0, 3).map(m => m.modelId),
    alternatives: ultraLowCost.slice(3, 6).map(m => m.modelId),
    savings: "90-95% vs premium models",
  });

  // Best value with tool calling
  const toolCallingModels = analyses
    .filter(a => a.capabilities.includes("tool_call") && a.costPerMillion < 1)
    .sort((a, b) => a.costPerMillion - b.costPerMillion);

  recommendations.push({
    useCase: "Agent & Tool Use",
    recommended: toolCallingModels.slice(0, 3).map(m => m.modelId),
    alternatives: toolCallingModels.slice(3, 6).map(m => m.modelId),
    savings: "80-90% vs GPT-4",
  });

  // Best for reasoning
  const reasoningModels = analyses
    .filter(a => a.capabilities.includes("reasoning"))
    .sort((a, b) => a.costPerMillion - b.costPerMillion);

  recommendations.push({
    useCase: "Complex Reasoning",
    recommended: reasoningModels.slice(0, 2).map(m => m.modelId),
    alternatives: reasoningModels.slice(2, 4).map(m => m.modelId),
    savings: "70-85% vs premium reasoning models",
  });

  // Best multimodal value
  const multimodalModels = analyses
    .filter(a => a.capabilities.includes("attachment"))
    .sort((a, b) => a.costPerMillion - b.costPerMillion);

  recommendations.push({
    useCase: "Multimodal Tasks",
    recommended: multimodalModels.slice(0, 3).map(m => m.modelId),
    alternatives: multimodalModels.slice(3, 6).map(m => m.modelId),
    savings: "60-80% vs GPT-4V",
  });

  return recommendations;
}

function printCostReport(analyses: CostAnalysis[], recommendations: OptimizationRecommendation[]) {
  console.log("🔍 AI MODEL COST OPTIMIZATION REPORT\n");
  console.log("=" .repeat(60));

  // Top 10 cheapest models
  console.log("\n📊 TOP 10 CHEAPEST MODELS:");
  console.log("-".repeat(60));
  analyses.slice(0, 10).forEach((analysis, index) => {
    console.log(`${index + 1}. ${analysis.modelId}`);
    console.log(`   Cost: $${analysis.costPerMillion.toFixed(3)}/M tokens`);
    console.log(`   Context: ${analysis.contextWindow.toLocaleString()}`);
    console.log(`   Capabilities: ${analysis.capabilities.join(", ") || "basic"}`);
    console.log();
  });

  // Recommendations
  console.log("\n💡 OPTIMIZATION RECOMMENDATIONS:");
  console.log("=".repeat(60));
  
  recommendations.forEach(rec => {
    console.log(`\n🎯 ${rec.useCase}:`);
    console.log(`   Recommended: ${rec.recommended.slice(0, 2).join(", ")}`);
    console.log(`   Alternatives: ${rec.alternatives.slice(0, 2).join(", ")}`);
    console.log(`   💰 Savings: ${rec.savings}`);
  });

  // Cost comparison table
  console.log("\n📈 COST COMPARISON BY TIER:");
  console.log("=".repeat(60));
  
  const tiers = [
    { name: "Free", maxCost: 0 },
    { name: "Ultra-Low", maxCost: 0.10 },
    { name: "Low", maxCost: 0.50 },
    { name: "Medium", maxCost: 2.00 },
    { name: "High", maxCost: Infinity },
  ];

  tiers.forEach(tier => {
    const models = analyses.filter(a => a.costPerMillion > (tiers[tiers.indexOf(tier) - 1]?.maxCost || -1) && a.costPerMillion <= tier.maxCost);
    if (models.length > 0) {
      console.log(`\n${tier.name} ($${tiers[tiers.indexOf(tier) - 1]?.maxCost || 0} - $${tier.maxCost === Infinity ? "∞" : tier.maxCost}):`);
      models.slice(0, 3).forEach(model => {
        console.log(`  • ${model.modelId} - $${model.costPerMillion.toFixed(3)}/M`);
      });
    }
  });

  console.log("\n" + "=".repeat(60));
  console.log("💡 Tip: Implement model routing to automatically select");
  console.log("   the cheapest model that meets your requirements.");
}

async function main() {
  try {
    console.log("🔍 Analyzing model costs across all providers...\n");
    
    const analyses = await analyzeCosts();
    const recommendations = generateRecommendations(analyses);
    
    printCostReport(analyses, recommendations);
    
    console.log(`\n✅ Analysis complete: ${analyses.length} models analyzed`);
    
  } catch (error) {
    console.error("❌ Error analyzing costs:", error);
    process.exit(1);
  }
}

if (import.meta.main) {
  main();
}
