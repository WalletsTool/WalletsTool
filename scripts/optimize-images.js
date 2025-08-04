import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const avatarDir = path.join(__dirname, '../public/avatar');
const outputDir = path.join(__dirname, '../public/avatar/optimized');

// 确保输出目录存在
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

async function optimizeImages() {
  const files = fs.readdirSync(avatarDir).filter(file => file.endsWith('.png'));
  
  for (const file of files) {
    const inputPath = path.join(avatarDir, file);
    const outputPath = path.join(outputDir, file.replace('.png', '.webp'));
    
    try {
      const stats = fs.statSync(inputPath);
      console.log(`优化 ${file} (原始大小: ${(stats.size / 1024).toFixed(2)}KB)`);
      
      await sharp(inputPath)
        .webp({ quality: 80, effort: 6 })
        .toFile(outputPath);
      
      const newStats = fs.statSync(outputPath);
      const reduction = ((stats.size - newStats.size) / stats.size * 100).toFixed(2);
      console.log(`✓ ${file} -> ${file.replace('.png', '.webp')} (新大小: ${(newStats.size / 1024).toFixed(2)}KB, 减少: ${reduction}%)`);
    } catch (error) {
      console.error(`优化 ${file} 时出错:`, error);
    }
  }
}

optimizeImages().then(() => {
  console.log('\n图片优化完成！');
}).catch(console.error);