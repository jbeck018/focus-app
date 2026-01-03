// components/ai-settings/AISettingsDialog.tsx
// Main AI settings dialog with provider configuration

import * as React from "react";
import { AlertTriangle, Loader2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { toast } from "sonner";

import type { ProviderType, ProviderConfig, ModelInfo } from "@/hooks/useAIProviders";
import {
  useAIProviderManager,
  useAvailableModels,
  useSaveApiKey,
  useDownloadModel,
  useModelDownloadStatus,
  useModelDownloadProgress,
} from "@/hooks/useAIProviders";

import { APIKeyInput } from "./APIKeyInput";
import { ModelDownloadProgress } from "./ModelDownloadProgress";

export interface AISettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Recommended models for each provider
 */
const RECOMMENDED_MODELS: Record<ProviderType, ModelInfo[]> = {
  local: [
    {
      id: "phi-3.5-mini",
      name: "Phi-3.5-mini",
      description: "Recommended - Fast and capable",
      sizeMb: 2300,
      recommended: true,
    },
    {
      id: "tinyllama",
      name: "TinyLlama",
      description: "Faster and smaller",
      sizeMb: 670,
    },
  ],
  openai: [
    {
      id: "gpt-4o",
      name: "GPT-4o",
      description: "Best quality, latest model",
      recommended: true,
    },
    {
      id: "gpt-4o-mini",
      name: "GPT-4o Mini",
      description: "Faster and cheaper",
    },
    {
      id: "gpt-3.5-turbo",
      name: "GPT-3.5 Turbo",
      description: "Legacy model",
    },
  ],
  anthropic: [
    {
      id: "claude-3-5-sonnet-latest",
      name: "Claude 3.5 Sonnet",
      description: "Best quality, most capable",
      recommended: true,
    },
    {
      id: "claude-3-haiku-20240307",
      name: "Claude 3 Haiku",
      description: "Faster and cheaper",
    },
  ],
  google: [
    {
      id: "gemini-1.5-pro",
      name: "Gemini 1.5 Pro",
      description: "Best quality",
      recommended: true,
    },
    {
      id: "gemini-1.5-flash",
      name: "Gemini 1.5 Flash",
      description: "Faster",
    },
  ],
  openrouter: [],
};

/**
 * AISettingsDialog Component
 *
 * Main settings dialog for AI provider configuration with:
 * - Provider selection tabs (Local, OpenAI, Anthropic, Google, OpenRouter)
 * - Model selection dropdowns
 * - API key input for cloud providers
 * - Model download for local provider
 * - Save/Cancel actions
 *
 * @example
 * ```tsx
 * const [open, setOpen] = useState(false);
 *
 * return (
 *   <>
 *     <Button onClick={() => setOpen(true)}>AI Settings</Button>
 *     <AISettingsDialog open={open} onOpenChange={setOpen} />
 *   </>
 * );
 * ```
 */
export function AISettingsDialog({ open, onOpenChange }: AISettingsDialogProps) {
  const { activeProvider, setProvider, isLoading } = useAIProviderManager();
  const saveApiKey = useSaveApiKey();
  const downloadModel = useDownloadModel();
  const { data: downloadStatus } = useModelDownloadStatus();

  // Local state for form
  const [selectedProvider, setSelectedProvider] = React.useState<ProviderType>("local");
  const [selectedModel, setSelectedModel] = React.useState<string>("");
  const [apiKey, setApiKey] = React.useState<string>("");
  const [showCloudWarning, setShowCloudWarning] = React.useState(false);
  const [isSaving, setIsSaving] = React.useState(false);

  // Fetch available models for selected provider
  const { data: availableModels, isLoading: modelsLoading } = useAvailableModels(
    selectedProvider,
    { enabled: open }
  );

  // Merge API models with recommended models
  const models = React.useMemo(() => {
    const recommended = RECOMMENDED_MODELS[selectedProvider] || [];
    const fetched = availableModels || [];

    // For OpenRouter, use fetched models exclusively
    if (selectedProvider === "openrouter") {
      return fetched;
    }

    // For others, merge and deduplicate
    const merged = [...recommended];
    fetched.forEach((model) => {
      if (!merged.find((m) => m.id === model.id)) {
        merged.push(model);
      }
    });

    return merged;
  }, [selectedProvider, availableModels]);

  // Initialize form when dialog opens
  React.useEffect(() => {
    if (open && activeProvider) {
      setSelectedProvider(activeProvider.provider as ProviderType);

      // Extract model based on provider type
      const model = activeProvider.provider === "local"
        ? (activeProvider as any).model_path
        : (activeProvider as any).model;

      setSelectedModel(model || "");
      setApiKey(""); // Don't prefill API key for security
    } else if (open && !activeProvider) {
      setSelectedProvider("local");
      setSelectedModel("");
      setApiKey("");
    }
  }, [open, activeProvider]);

  // Auto-select first model if none selected
  React.useEffect(() => {
    if (models.length > 0 && !selectedModel) {
      const recommended = models.find((m) => m.recommended);
      setSelectedModel(recommended?.id || models[0].id);
    }
  }, [models, selectedModel]);

  // Listen to download events and show toasts
  useModelDownloadProgress(
    undefined, // Don't need progress callback - handled by query cache
    (modelName) => {
      toast.success(`Successfully downloaded ${modelName}`);
    },
    (error, modelName) => {
      toast.error(`Download failed${modelName ? ` for ${modelName}` : ""}: ${error}`);
    }
  );

  const handleProviderChange = React.useCallback((provider: ProviderType) => {
    setSelectedProvider(provider);
    setSelectedModel("");
    setApiKey("");

    // Show warning when switching from local to cloud
    const wasLocal = selectedProvider === "local";
    const isCloud = provider !== "local";
    if (wasLocal && isCloud) {
      setShowCloudWarning(true);
    } else {
      setShowCloudWarning(false);
    }
  }, [selectedProvider]);

  const handleSave = React.useCallback(async () => {
    if (!selectedModel) {
      toast.error("Please select a model");
      return;
    }

    setIsSaving(true);

    try {
      // For cloud providers, save API key first if provided
      if (selectedProvider !== "local" && apiKey.trim()) {
        await saveApiKey.mutateAsync({ provider: selectedProvider, apiKey });
      }

      // Build the provider config based on the backend's expected structure
      let config: ProviderConfig;

      switch (selectedProvider) {
        case "openai":
          config = {
            provider: "openai",
            api_key: apiKey.trim(),
            model: selectedModel,
          };
          break;
        case "anthropic":
          config = {
            provider: "anthropic",
            api_key: apiKey.trim(),
            model: selectedModel,
          };
          break;
        case "google":
          config = {
            provider: "google",
            api_key: apiKey.trim(),
            model: selectedModel,
          };
          break;
        case "openrouter":
          config = {
            provider: "openrouter",
            api_key: apiKey.trim(),
            model: selectedModel,
            app_name: "FocusFlow",
          };
          break;
        case "local":
          config = {
            provider: "local",
            model_path: selectedModel,
          };
          break;
        default:
          throw new Error(`Unsupported provider: ${selectedProvider}`);
      }

      await setProvider.mutateAsync(config);

      toast.success("AI provider updated successfully");
      onOpenChange(false);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to save settings";
      toast.error(message);
    } finally {
      setIsSaving(false);
    }
  }, [
    selectedProvider,
    selectedModel,
    apiKey,
    saveApiKey,
    setProvider,
    onOpenChange,
  ]);

  const handleCancel = React.useCallback(() => {
    onOpenChange(false);
    setShowCloudWarning(false);
  }, [onOpenChange]);

  const requiresApiKey = selectedProvider !== "local";
  const canSave =
    selectedModel &&
    (!requiresApiKey || apiKey.trim().length > 0) &&
    !isSaving &&
    !downloadStatus?.isDownloading;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>AI Settings</DialogTitle>
          <DialogDescription>
            Choose your AI provider and configure model settings. Your preferences are saved
            locally.
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <Tabs value={selectedProvider} onValueChange={(v) => handleProviderChange(v as ProviderType)} className="mt-4">
            <TabsList className="grid w-full grid-cols-5">
              <TabsTrigger value="local">Local</TabsTrigger>
              <TabsTrigger value="openai">OpenAI</TabsTrigger>
              <TabsTrigger value="anthropic">Anthropic</TabsTrigger>
              <TabsTrigger value="google">Google</TabsTrigger>
              <TabsTrigger value="openrouter">OpenRouter</TabsTrigger>
            </TabsList>

            {/* Cloud provider warning */}
            {showCloudWarning && (
              <Alert variant="default" className="mt-4">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Privacy Notice</AlertTitle>
                <AlertDescription>
                  Cloud providers send your data to external servers. Local models keep
                  everything private on your device.
                </AlertDescription>
              </Alert>
            )}

            {/* Local Provider */}
            <TabsContent value="local" className="space-y-4 mt-4">
              <div className="space-y-2">
                <Label htmlFor="local-model">Model</Label>
                <Select
                  value={selectedModel}
                  onValueChange={setSelectedModel}
                  disabled={downloadStatus?.isDownloading}
                >
                  <SelectTrigger id="local-model">
                    <SelectValue placeholder="Select a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((model) => (
                      <SelectItem key={model.id} value={model.id}>
                        <div className="flex items-center justify-between gap-2">
                          <span>{model.name}</span>
                          {model.recommended && (
                            <span className="text-xs text-primary">(Recommended)</span>
                          )}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {selectedModel && (
                  <p className="text-sm text-muted-foreground">
                    {models.find((m) => m.id === selectedModel)?.description}
                    {models.find((m) => m.id === selectedModel)?.sizeMb &&
                      ` - ${(models.find((m) => m.id === selectedModel)!.sizeMb! / 1024).toFixed(1)} GB`}
                  </p>
                )}
              </div>

              {/* Show download progress or error */}
              {(downloadStatus?.isDownloading || downloadStatus?.error) && (
                <ModelDownloadProgress
                  progress={downloadStatus.progress}
                  error={downloadStatus.error}
                  isDownloading={downloadStatus.isDownloading}
                  modelName={downloadStatus.modelName || selectedModel}
                  onRetry={async () => {
                    if (downloadStatus.modelName || selectedModel) {
                      try {
                        await downloadModel.mutateAsync(downloadStatus.modelName || selectedModel);
                        toast.success("Model download restarted");
                      } catch (error) {
                        const message = error instanceof Error ? error.message : "Failed to restart download";
                        toast.error(message);
                      }
                    }
                  }}
                />
              )}

              {/* Show download button when not downloading and no error */}
              {selectedModel && !downloadStatus?.isDownloading && !downloadStatus?.error && (
                <Button
                  type="button"
                  variant="outline"
                  className="w-full"
                  onClick={async () => {
                    try {
                      await downloadModel.mutateAsync(selectedModel);
                      toast.success("Model download started");
                    } catch (error) {
                      const message = error instanceof Error ? error.message : "Failed to start download";
                      toast.error(message);
                    }
                  }}
                  disabled={downloadModel.isPending}
                >
                  {downloadModel.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Starting Download...
                    </>
                  ) : (
                    "Download Model"
                  )}
                </Button>
              )}

              <Alert>
                <AlertDescription className="text-xs">
                  Local models run privately on your device. First-time setup requires downloading
                  the model.
                </AlertDescription>
              </Alert>
            </TabsContent>

            {/* OpenAI Provider */}
            <TabsContent value="openai" className="space-y-4 mt-4">
              <APIKeyInput
                provider="openai"
                modelId={selectedModel}
                value={apiKey}
                onChange={setApiKey}
                placeholder="sk-..."
                onTestSuccess={() => toast.success("OpenAI connection successful")}
                onTestError={(error) => toast.error(error)}
              />

              <Separator />

              <div className="space-y-2">
                <Label htmlFor="openai-model">Model</Label>
                <Select
                  value={selectedModel}
                  onValueChange={setSelectedModel}
                  disabled={modelsLoading}
                >
                  <SelectTrigger id="openai-model">
                    <SelectValue placeholder="Select a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((model) => (
                      <SelectItem key={model.id} value={model.id}>
                        {model.name}
                        {model.recommended && " (Recommended)"}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {selectedModel && (
                  <p className="text-sm text-muted-foreground">
                    {models.find((m) => m.id === selectedModel)?.description}
                  </p>
                )}
              </div>
            </TabsContent>

            {/* Anthropic Provider */}
            <TabsContent value="anthropic" className="space-y-4 mt-4">
              <APIKeyInput
                provider="anthropic"
                modelId={selectedModel}
                value={apiKey}
                onChange={setApiKey}
                placeholder="sk-ant-..."
                onTestSuccess={() => toast.success("Anthropic connection successful")}
                onTestError={(error) => toast.error(error)}
              />

              <Separator />

              <div className="space-y-2">
                <Label htmlFor="anthropic-model">Model</Label>
                <Select
                  value={selectedModel}
                  onValueChange={setSelectedModel}
                  disabled={modelsLoading}
                >
                  <SelectTrigger id="anthropic-model">
                    <SelectValue placeholder="Select a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((model) => (
                      <SelectItem key={model.id} value={model.id}>
                        {model.name}
                        {model.recommended && " (Recommended)"}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {selectedModel && (
                  <p className="text-sm text-muted-foreground">
                    {models.find((m) => m.id === selectedModel)?.description}
                  </p>
                )}
              </div>
            </TabsContent>

            {/* Google Provider */}
            <TabsContent value="google" className="space-y-4 mt-4">
              <APIKeyInput
                provider="google"
                modelId={selectedModel}
                value={apiKey}
                onChange={setApiKey}
                placeholder="AIza..."
                onTestSuccess={() => toast.success("Google AI connection successful")}
                onTestError={(error) => toast.error(error)}
              />

              <Separator />

              <div className="space-y-2">
                <Label htmlFor="google-model">Model</Label>
                <Select
                  value={selectedModel}
                  onValueChange={setSelectedModel}
                  disabled={modelsLoading}
                >
                  <SelectTrigger id="google-model">
                    <SelectValue placeholder="Select a model" />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((model) => (
                      <SelectItem key={model.id} value={model.id}>
                        {model.name}
                        {model.recommended && " (Recommended)"}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {selectedModel && (
                  <p className="text-sm text-muted-foreground">
                    {models.find((m) => m.id === selectedModel)?.description}
                  </p>
                )}
              </div>
            </TabsContent>

            {/* OpenRouter Provider */}
            <TabsContent value="openrouter" className="space-y-4 mt-4">
              <APIKeyInput
                provider="openrouter"
                modelId={selectedModel}
                value={apiKey}
                onChange={setApiKey}
                placeholder="sk-or-..."
                onTestSuccess={() => toast.success("OpenRouter connection successful")}
                onTestError={(error) => toast.error(error)}
              />

              <Separator />

              <div className="space-y-2">
                <Label htmlFor="openrouter-model">Model</Label>
                <Select
                  value={selectedModel}
                  onValueChange={setSelectedModel}
                  disabled={modelsLoading}
                >
                  <SelectTrigger id="openrouter-model">
                    <SelectValue
                      placeholder={modelsLoading ? "Loading models..." : "Select a model"}
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {models.length === 0 && !modelsLoading ? (
                      <SelectItem value="none" disabled>
                        No models available
                      </SelectItem>
                    ) : (
                      models.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          {model.name}
                        </SelectItem>
                      ))
                    )}
                  </SelectContent>
                </Select>
                {selectedModel && (
                  <p className="text-sm text-muted-foreground">
                    {models.find((m) => m.id === selectedModel)?.description}
                  </p>
                )}
              </div>

              <Alert>
                <AlertDescription className="text-xs">
                  OpenRouter provides access to multiple AI models through a single API.
                </AlertDescription>
              </Alert>
            </TabsContent>
          </Tabs>
        )}

        <DialogFooter className="mt-6">
          <Button type="button" variant="outline" onClick={handleCancel}>
            Cancel
          </Button>
          <Button type="button" onClick={handleSave} disabled={!canSave}>
            {isSaving ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              "Save Settings"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default AISettingsDialog;
