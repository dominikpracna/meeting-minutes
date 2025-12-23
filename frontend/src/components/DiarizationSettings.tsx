import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/button';
import { Label } from './ui/label';
import { Download, CheckCircle2, RefreshCw } from 'lucide-react';
import { toast } from 'sonner';

interface DiarizationSettingsProps {
    autoCheck?: boolean;
}

export function DiarizationSettings({ autoCheck = true }: DiarizationSettingsProps) {
    const [isDownloaded, setIsDownloaded] = useState<boolean>(false);
    const [isDownloading, setIsDownloading] = useState<boolean>(false);
    const [downloadProgress, setDownloadProgress] = useState<number>(0);
    const [checkStatus, setCheckStatus] = useState<boolean>(false);

    useEffect(() => {
        // We can check if model exists by trying to download it - the backend command
        // checks existence first and returns immediately if exists.
        // Ideally we should have a `diarization_check_model` command but `diarization_download_model` works safely.

        const checkModel = async () => {
             // For now we assume if we didn't get a "download complete" event it might not be there.
             // But actually the backend command emits 'diarization-model-download-complete' even if it exists.
             // So we can just listen to events.
             // However, to know initial state, we can't easily query without a specific command.
             // I will implement a quick check logic: Call download, if it returns instantly it's there.
             // But that might trigger a download if not there.
             // So I will just provide the download button.
             // If user clicks, it downloads or confirms.
        };

        if (autoCheck) {
            // Setup listeners
             const setupListeners = async () => {
                const { listen } = await import('@tauri-apps/api/event');

                const unlistenProgress = await listen<number>('diarization-model-download-progress', (event) => {
                    setDownloadProgress(event.payload);
                    setIsDownloading(true);
                });

                const unlistenComplete = await listen('diarization-model-download-complete', () => {
                    setIsDownloading(false);
                    setIsDownloaded(true);
                    setDownloadProgress(100);
                    toast.success("Diarization model ready");
                });

                return () => {
                    unlistenProgress();
                    unlistenComplete();
                };
            };

            setupListeners();
        }
    }, [autoCheck]);

    const handleDownload = async () => {
        setIsDownloading(true);
        try {
            toast.info("Checking/Downloading Diarization model...");
            await invoke('diarization_download_model');
        } catch (error) {
            console.error("Failed to download diarization model:", error);
            setIsDownloading(false);
            toast.error("Failed to download diarization model");
        }
    };

    return (
        <div className="border rounded-md p-4 bg-gray-50 mt-4">
            <div className="flex justify-between items-center mb-2">
                <Label className="text-sm font-semibold text-gray-900">Speaker Diarization</Label>
                {isDownloaded && <span className="text-xs text-green-600 font-medium flex items-center"><CheckCircle2 className="w-3 h-3 mr-1"/> Ready</span>}
            </div>

            <p className="text-xs text-gray-500 mb-3">
                Enable speaker identification (Speaker 1, Speaker 2...) by downloading the embedding model (~30MB).
            </p>

            {!isDownloaded ? (
                <div className="space-y-2">
                    <Button
                        onClick={handleDownload}
                        disabled={isDownloading}
                        variant="outline"
                        size="sm"
                        className="w-full"
                    >
                        {isDownloading ? (
                            <>
                                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                                {downloadProgress > 0 ? `Downloading ${downloadProgress}%` : "Downloading..."}
                            </>
                        ) : (
                            <>
                                <Download className="mr-2 h-4 w-4" />
                                Download Model
                            </>
                        )}
                    </Button>

                    {isDownloading && (
                         <div className="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
                            <div
                                className="h-full bg-blue-500 transition-all duration-300"
                                style={{ width: `${downloadProgress}%` }}
                            />
                        </div>
                    )}
                </div>
            ) : (
                 <Button
                    variant="ghost"
                    size="sm"
                    className="w-full text-green-600 hover:text-green-700 hover:bg-green-50 pointer-events-none"
                >
                    <CheckCircle2 className="mr-2 h-4 w-4" />
                    Model Installed
                </Button>
            )}
        </div>
    );
}
