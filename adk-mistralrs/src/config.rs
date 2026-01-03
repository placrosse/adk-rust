//! Configuration types for mistral.rs model loading.

use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for mistral.rs model loading.
#[derive(Debug, Clone)]
pub struct MistralRsConfig {
    /// Model source: HuggingFace ID, local path, GGUF, or UQFF path
    pub model_source: ModelSource,

    /// Model architecture type
    pub architecture: ModelArchitecture,

    /// Data type for model weights
    pub dtype: DataType,

    /// Device configuration
    pub device: DeviceConfig,

    /// ISQ quantization settings (optional)
    pub isq: Option<IsqConfig>,

    /// LoRA/X-LoRA adapter configuration (optional)
    pub adapter: Option<AdapterConfig>,

    /// Generation defaults - temperature
    pub temperature: Option<f32>,

    /// Generation defaults - top_p
    pub top_p: Option<f32>,

    /// Generation defaults - top_k
    pub top_k: Option<i32>,

    /// Generation defaults - max_tokens
    pub max_tokens: Option<i32>,

    /// Context length
    pub num_ctx: Option<usize>,

    /// Enable PagedAttention
    pub paged_attention: bool,

    /// Topology file for per-layer config (optional)
    pub topology_path: Option<PathBuf>,

    /// Custom chat template (optional)
    pub chat_template: Option<String>,

    /// Custom tokenizer path (optional)
    pub tokenizer_path: Option<PathBuf>,

    /// MatFormer configuration for Gemma 3n (optional)
    pub matformer: Option<MatFormerConfig>,

    /// MCP client configuration for external tools (optional)
    pub mcp_config: Option<PathBuf>,
}

impl Default for MistralRsConfig {
    fn default() -> Self {
        Self {
            model_source: ModelSource::HuggingFace(String::new()),
            architecture: ModelArchitecture::default(),
            dtype: DataType::default(),
            device: DeviceConfig::default(),
            isq: None,
            adapter: None,
            temperature: None,
            top_p: None,
            top_k: None,
            max_tokens: None,
            num_ctx: None,
            paged_attention: false,
            topology_path: None,
            chat_template: None,
            tokenizer_path: None,
            matformer: None,
            mcp_config: None,
        }
    }
}

impl MistralRsConfig {
    /// Create a new config builder
    pub fn builder() -> MistralRsConfigBuilder {
        MistralRsConfigBuilder::default()
    }
}

/// Builder for MistralRsConfig
#[derive(Debug, Clone, Default)]
pub struct MistralRsConfigBuilder {
    config: MistralRsConfig,
}

impl MistralRsConfigBuilder {
    /// Set the model source
    pub fn model_source(mut self, source: ModelSource) -> Self {
        self.config.model_source = source;
        self
    }

    /// Set the model architecture
    pub fn architecture(mut self, arch: ModelArchitecture) -> Self {
        self.config.architecture = arch;
        self
    }

    /// Set the data type
    pub fn dtype(mut self, dtype: DataType) -> Self {
        self.config.dtype = dtype;
        self
    }

    /// Set the device configuration
    pub fn device(mut self, device: DeviceConfig) -> Self {
        self.config.device = device;
        self
    }

    /// Enable ISQ quantization
    pub fn isq(mut self, level: QuantizationLevel) -> Self {
        self.config.isq = Some(IsqConfig {
            level,
            layer_overrides: None,
        });
        self
    }

    /// Set ISQ configuration with layer overrides
    pub fn isq_config(mut self, config: IsqConfig) -> Self {
        self.config.isq = Some(config);
        self
    }

    /// Set adapter configuration
    pub fn adapter(mut self, adapter: AdapterConfig) -> Self {
        self.config.adapter = Some(adapter);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = Some(temp);
        self
    }

    /// Set top_p
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.config.top_p = Some(top_p);
        self
    }

    /// Set top_k
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.config.top_k = Some(top_k);
        self
    }

    /// Set max_tokens
    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.config.max_tokens = Some(max_tokens);
        self
    }

    /// Set context length
    pub fn num_ctx(mut self, num_ctx: usize) -> Self {
        self.config.num_ctx = Some(num_ctx);
        self
    }

    /// Enable PagedAttention
    pub fn paged_attention(mut self, enabled: bool) -> Self {
        self.config.paged_attention = enabled;
        self
    }

    /// Set topology file path
    pub fn topology_path(mut self, path: PathBuf) -> Self {
        self.config.topology_path = Some(path);
        self
    }

    /// Set custom chat template
    pub fn chat_template(mut self, template: String) -> Self {
        self.config.chat_template = Some(template);
        self
    }

    /// Set custom tokenizer path
    pub fn tokenizer_path(mut self, path: PathBuf) -> Self {
        self.config.tokenizer_path = Some(path);
        self
    }

    /// Set MatFormer configuration
    pub fn matformer(mut self, config: MatFormerConfig) -> Self {
        self.config.matformer = Some(config);
        self
    }

    /// Set MCP client configuration path
    pub fn mcp_config(mut self, path: PathBuf) -> Self {
        self.config.mcp_config = Some(path);
        self
    }

    /// Build the configuration
    pub fn build(self) -> MistralRsConfig {
        self.config
    }
}

/// Model source specification
#[derive(Debug, Clone)]
pub enum ModelSource {
    /// HuggingFace Hub model ID (e.g., "mistralai/Mistral-7B-v0.1")
    HuggingFace(String),
    /// Local directory path
    Local(PathBuf),
    /// GGUF file path
    Gguf(PathBuf),
    /// UQFF pre-quantized file path
    Uqff(PathBuf),
}

impl ModelSource {
    /// Create a HuggingFace model source
    pub fn huggingface(model_id: impl Into<String>) -> Self {
        Self::HuggingFace(model_id.into())
    }

    /// Create a local path model source
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self::Local(path.into())
    }

    /// Create a GGUF file model source
    pub fn gguf(path: impl Into<PathBuf>) -> Self {
        Self::Gguf(path.into())
    }

    /// Create a UQFF file model source
    pub fn uqff(path: impl Into<PathBuf>) -> Self {
        Self::Uqff(path.into())
    }
}

/// Model architecture type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ModelArchitecture {
    /// Plain text model
    #[default]
    Plain,
    /// Vision-language model
    Vision,
    /// Diffusion model for image generation
    Diffusion,
    /// Speech generation model
    Speech,
    /// Embedding model
    Embedding,
    /// X-LoRA adapter model
    XLora,
    /// LoRA adapter model
    Lora,
}

/// Data type for model weights
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DataType {
    /// 32-bit floating point
    F32,
    /// 16-bit floating point
    F16,
    /// Brain floating point 16
    BF16,
    /// Auto-detect based on model and hardware
    #[default]
    Auto,
}

/// Device configuration
#[derive(Debug, Clone, Default)]
pub struct DeviceConfig {
    /// Primary device
    pub device: Device,
    /// Device mapping for multi-device (layer name -> device)
    pub device_map: Option<HashMap<String, Device>>,
}

impl DeviceConfig {
    /// Create a new device config with the specified device
    pub fn new(device: Device) -> Self {
        Self {
            device,
            device_map: None,
        }
    }

    /// Create a device config with device mapping
    pub fn with_map(device: Device, device_map: HashMap<String, Device>) -> Self {
        Self {
            device,
            device_map: Some(device_map),
        }
    }
}

/// Device selection
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Device {
    /// Auto-detect best available device
    #[default]
    Auto,
    /// CPU
    Cpu,
    /// CUDA GPU with index
    Cuda(usize),
    /// Apple Metal
    Metal,
}

/// ISQ (In-Situ Quantization) configuration
#[derive(Debug, Clone)]
pub struct IsqConfig {
    /// Quantization level
    pub level: QuantizationLevel,
    /// Per-layer overrides (optional)
    pub layer_overrides: Option<HashMap<String, QuantizationLevel>>,
}

impl IsqConfig {
    /// Create a new ISQ config with the specified level
    pub fn new(level: QuantizationLevel) -> Self {
        Self {
            level,
            layer_overrides: None,
        }
    }

    /// Create an ISQ config with layer overrides
    pub fn with_overrides(
        level: QuantizationLevel,
        overrides: HashMap<String, QuantizationLevel>,
    ) -> Self {
        Self {
            level,
            layer_overrides: Some(overrides),
        }
    }
}

/// Quantization level for ISQ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationLevel {
    /// 4-bit quantization (variant 0)
    Q4_0,
    /// 4-bit quantization (variant 1)
    Q4_1,
    /// 5-bit quantization (variant 0)
    Q5_0,
    /// 5-bit quantization (variant 1)
    Q5_1,
    /// 8-bit quantization (variant 0)
    Q8_0,
    /// 8-bit quantization (variant 1)
    Q8_1,
    /// 2-bit K-quant
    Q2K,
    /// 3-bit K-quant
    Q3K,
    /// 4-bit K-quant
    Q4K,
    /// 5-bit K-quant
    Q5K,
    /// 6-bit K-quant
    Q6K,
}

/// LoRA/X-LoRA adapter configuration
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    /// Adapter type
    pub adapter_type: AdapterType,
    /// Path to adapter weights or HuggingFace ID
    pub adapter_source: String,
    /// Adapter ordering file (for X-LoRA)
    pub ordering: Option<PathBuf>,
}

impl AdapterConfig {
    /// Create a LoRA adapter config
    pub fn lora(source: impl Into<String>) -> Self {
        Self {
            adapter_type: AdapterType::LoRA,
            adapter_source: source.into(),
            ordering: None,
        }
    }

    /// Create an X-LoRA adapter config
    pub fn xlora(source: impl Into<String>, ordering: PathBuf) -> Self {
        Self {
            adapter_type: AdapterType::XLoRA,
            adapter_source: source.into(),
            ordering: Some(ordering),
        }
    }
}

/// Adapter type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterType {
    /// Standard LoRA adapter
    LoRA,
    /// X-LoRA with dynamic adapter mixing
    XLoRA,
}

/// MatFormer configuration for Gemma 3n models
#[derive(Debug, Clone)]
pub struct MatFormerConfig {
    /// Target model size (e.g., "2b", "4b")
    pub target_size: String,
}

impl MatFormerConfig {
    /// Create a new MatFormer config
    pub fn new(target_size: impl Into<String>) -> Self {
        Self {
            target_size: target_size.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = MistralRsConfig::builder()
            .model_source(ModelSource::huggingface("test/model"))
            .architecture(ModelArchitecture::Plain)
            .dtype(DataType::Auto)
            .temperature(0.7)
            .top_p(0.9)
            .max_tokens(1024)
            .paged_attention(true)
            .build();

        assert!(matches!(config.model_source, ModelSource::HuggingFace(_)));
        assert_eq!(config.architecture, ModelArchitecture::Plain);
        assert_eq!(config.dtype, DataType::Auto);
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.top_p, Some(0.9));
        assert_eq!(config.max_tokens, Some(1024));
        assert!(config.paged_attention);
    }

    #[test]
    fn test_model_source_variants() {
        let hf = ModelSource::huggingface("org/model");
        assert!(matches!(hf, ModelSource::HuggingFace(_)));

        let local = ModelSource::local("/path/to/model");
        assert!(matches!(local, ModelSource::Local(_)));

        let gguf = ModelSource::gguf("/path/to/model.gguf");
        assert!(matches!(gguf, ModelSource::Gguf(_)));

        let uqff = ModelSource::uqff("/path/to/model.uqff");
        assert!(matches!(uqff, ModelSource::Uqff(_)));
    }

    #[test]
    fn test_all_architecture_variants() {
        let variants = [
            ModelArchitecture::Plain,
            ModelArchitecture::Vision,
            ModelArchitecture::Diffusion,
            ModelArchitecture::Speech,
            ModelArchitecture::Embedding,
            ModelArchitecture::XLora,
            ModelArchitecture::Lora,
        ];
        assert_eq!(variants.len(), 7);
    }

    #[test]
    fn test_all_dtype_variants() {
        let variants = [DataType::F32, DataType::F16, DataType::BF16, DataType::Auto];
        assert_eq!(variants.len(), 4);
    }

    #[test]
    fn test_all_device_variants() {
        let variants = [
            Device::Auto,
            Device::Cpu,
            Device::Cuda(0),
            Device::Cuda(1),
            Device::Metal,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_all_quantization_variants() {
        let variants = [
            QuantizationLevel::Q4_0,
            QuantizationLevel::Q4_1,
            QuantizationLevel::Q5_0,
            QuantizationLevel::Q5_1,
            QuantizationLevel::Q8_0,
            QuantizationLevel::Q8_1,
            QuantizationLevel::Q2K,
            QuantizationLevel::Q3K,
            QuantizationLevel::Q4K,
            QuantizationLevel::Q5K,
            QuantizationLevel::Q6K,
        ];
        assert_eq!(variants.len(), 11);
    }

    #[test]
    fn test_isq_config() {
        let config = IsqConfig::new(QuantizationLevel::Q4_0);
        assert_eq!(config.level, QuantizationLevel::Q4_0);
        assert!(config.layer_overrides.is_none());

        let mut overrides = HashMap::new();
        overrides.insert("layer1".to_string(), QuantizationLevel::Q8_0);
        let config_with_overrides =
            IsqConfig::with_overrides(QuantizationLevel::Q4_0, overrides);
        assert!(config_with_overrides.layer_overrides.is_some());
    }

    #[test]
    fn test_adapter_config() {
        let lora = AdapterConfig::lora("path/to/adapter");
        assert_eq!(lora.adapter_type, AdapterType::LoRA);
        assert!(lora.ordering.is_none());

        let xlora = AdapterConfig::xlora("path/to/adapter", PathBuf::from("ordering.json"));
        assert_eq!(xlora.adapter_type, AdapterType::XLoRA);
        assert!(xlora.ordering.is_some());
    }
}
