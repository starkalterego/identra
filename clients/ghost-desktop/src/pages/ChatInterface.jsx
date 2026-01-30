import { useState, useEffect, useRef } from "react";
import { 
  Send, 
  Search,
  FileText,
  User,
  Settings,
  X,
  Moon,
  Sun,
  Circle,
  Bell,
  Shield,
  LogOut,
  ChevronDown,
  Check
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

function ClaudeIcon({ className }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <rect x="3" y="3" width="18" height="18" rx="4" ry="4" />
      <circle cx="12" cy="12" r="4" />
    </svg>
  );
}
function GeminiIcon({ className }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M12 2L14.5 8.5L21 9L16 13.5L17.5 20L12 16.5L6.5 20L8 13.5L3 9L9.5 8.5L12 2Z" />
    </svg>
  );
}
function OpenAIIcon({ className }) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" className={className}>
      <path d="M12 2L4 8V16L12 22L20 16V8L12 2Z" />
    </svg>
  );
}
const modelIcons = { claude: ClaudeIcon, gemini: GeminiIcon, gpt: OpenAIIcon };

export default function ChatInterface() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const [selectedModel, setSelectedModel] = useState("claude"); // claude, gemini, gpt
  const [sessionInitialized, setSessionInitialized] = useState(false);
  const [conversationHistory, setConversationHistory] = useState([]);
  const [profileOpen, setProfileOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [themeOpen, setThemeOpen] = useState(false);
  const [theme, setTheme] = useState(() => {
    try {
      const stored = localStorage.getItem("identra-theme");
      if (stored === "light" || stored === "grey" || stored === "dark") return stored;
      return "dark";
    } catch {
      return "dark";
    }
  });
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);

  // Apply theme to document and persist
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    try {
      localStorage.setItem("identra-theme", theme);
    } catch (_) {}
  }, [theme]);

  const models = [
    { id: "claude", name: "Claude 3.5 Sonnet", color: "identra-claude" },
    { id: "gemini", name: "Gemini 1.5 Pro", color: "identra-gemini" },
    { id: "gpt", name: "GPT-4o", color: "identra-gpt" }
  ];

  const contextDocuments = [
    { id: 1, name: "Auth_Specs_v2.pdf", model: "claude", size: "2.4 MB" },
    { id: 2, name: "Security_Audit_2024", model: "gemini", size: "1.8 MB" },
    { id: 3, name: "Client_Meeting_Analysis", model: "gpt", size: "892 KB" }
  ];

  // Auto-initialize session on startup
  useEffect(() => {
    const initSession = async () => {
      try {
        await invoke("initialize_session");
        setSessionInitialized(true);
        console.log("✅ Session initialized - vault unlocked");
      } catch (err) {
        console.error("❌ Session initialization failed:", err);
        // Retry after 2 seconds
        setTimeout(initSession, 2000);
      }
    };
    initSession();
  }, []);

  useEffect(() => {
    // Load conversation history after session initialized
    if (sessionInitialized) {
      invoke("query_history", { limit: 50 })
        .then(history => {
          setConversationHistory(history);
          console.log("📜 Loaded", history.length, "conversations from database");
        })
        .catch(err => console.error("Failed to load history:", err));
    }
  }, [sessionInitialized]); // Re-fetch after session init

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || isProcessing) return;

    // Wait for session to be initialized
    if (!sessionInitialized) {
      console.warn("⏳ Waiting for session initialization...");
      return;
    }

    const userMessage = {
      id: Date.now(),
      role: "user",
      content: input,
      timestamp: new Date()
    };

    setMessages(prev => [...prev, userMessage]);
    setInput("");
    setIsProcessing(true);

    try {
      // Prepare conversation history for API
      // Note: Context limit is configured in .env (CHAT_CONTEXT_LIMIT, default: 10)
      const historyForAPI = messages.slice(-10).map(msg => ({
        role: msg.role,
        content: msg.content,
        timestamp: Math.floor(msg.timestamp.getTime() / 1000)
      }));

      // Call the new chat_with_ai command
      const response = await invoke("chat_with_ai", {
        message: input,
        model: selectedModel,
        conversationHistory: historyForAPI
      });
      
      const assistantMessage = {
        id: Date.now() + 1,
        role: "assistant",
        content: response.message,
        timestamp: new Date(),
        model: selectedModel
      };

      setMessages(prev => [...prev, assistantMessage]);
      
      // Refresh history to show the new conversation
      invoke("query_history", { limit: 50 })
        .then(history => {
          setConversationHistory(history);
        })
        .catch(err => console.error("Failed to refresh history:", err));
        
    } catch (err) {
      console.error("Error:", err);
      const errorMessage = {
        id: Date.now() + 1,
        role: "assistant",
        content: `Error: ${err}`,
        timestamp: new Date()
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // Auto-resize textarea based on content
  useEffect(() => {
    if (textareaRef.current) {
      // Reset height to auto to get the correct scrollHeight
      textareaRef.current.style.height = 'auto';
      // Set height to scrollHeight, but respect min and max
      const scrollHeight = textareaRef.current.scrollHeight;
      const minHeight = 80; // min-h-[80px]
      const maxHeight = 400; // max-h-[400px]
      const newHeight = Math.min(Math.max(scrollHeight, minHeight), maxHeight);
      textareaRef.current.style.height = `${newHeight}px`;
    }
  }, [input]);

  const handleLoadConversation = async (item) => {
    try {
      console.log("🔄 Loading conversation:", item.id);
      console.log("📦 Encrypted content:", item.content.substring(0, 50));
      
      // Decrypt the encrypted content
      const decryptedContent = await invoke("decrypt_memory", { encryptedVal: item.content });
      
      console.log("🔓 Decrypted content:", decryptedContent);
      
      // Parse the conversation JSON
      try {
        const conversation = JSON.parse(decryptedContent);
        
        // Load both user message and AI response
        const loadedMessages = [];
        
        if (conversation.user) {
          loadedMessages.push({
            id: `${item.id}-user`,
            role: "user",
            content: conversation.user,
            timestamp: new Date(conversation.timestamp || item.timestamp * 1000)
          });
        }
        
        if (conversation.assistant) {
          loadedMessages.push({
            id: `${item.id}-assistant`,
            role: "assistant",
            content: conversation.assistant,
            timestamp: new Date(conversation.timestamp || item.timestamp * 1000),
            model: conversation.model || selectedModel
          });
        }
        
        setMessages(loadedMessages);
        
        console.log("✅ Loaded conversation with", loadedMessages.length, "messages");
      } catch (error_) {
        console.warn("Not JSON format, treating as legacy:", error_.message);
        // If it's not JSON, treat it as a single user message (legacy format)
        const loadedMessage = {
          id: item.id,
          role: "user",
          content: decryptedContent,
          timestamp: new Date(item.timestamp * 1000)
        };
        
        setMessages([loadedMessage]);
        console.log("✅ Loaded legacy conversation:", item.id);
      }
      
      // Scroll to view
      setTimeout(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
      }, 100);
    } catch (err) {
      console.error("❌ Failed to decrypt conversation:", err);
      // Show error in UI
      setMessages([{
        id: Date.now(),
        role: "assistant",
        content: `Error loading conversation: ${err}`,
        timestamp: new Date()
      }]);
    }
  };

  const currentModel = models.find(m => m.id === selectedModel);

  return (
    <div className="flex h-screen bg-identra-bg text-identra-text-primary font-sans antialiased">
      
      {/* Left Section - Icon strip */}
      <aside className="w-14 bg-identra-surface/80 border-r border-identra-divider flex flex-col items-center py-4 gap-1 shrink-0">
        <button
          title="User Profile"
          onClick={() => {
            setSettingsOpen(false);
            setProfileOpen((v) => !v);
          }}
          className={`p-2.5 rounded-lg transition-colors ${profileOpen ? "text-identra-text-primary bg-identra-surface-elevated/80" : "text-identra-text-tertiary hover:text-identra-text-primary hover:bg-identra-surface-elevated/80"}`}
        >
          <User className="w-5 h-5" />
        </button>
        <div className="flex-1" />
        <button
          title="Settings"
          onClick={() => {
            setProfileOpen(false);
            setSettingsOpen((v) => !v);
          }}
          className={`p-2.5 rounded-lg transition-colors ${settingsOpen ? "text-identra-text-primary bg-identra-surface-elevated/80" : "text-identra-text-tertiary hover:text-identra-text-primary hover:bg-identra-surface-elevated/80"}`}
        >
          <Settings className="w-5 h-5" />
        </button>
      </aside>

      {/* Profile panel (includes pack details) */}
      {profileOpen && (
        <>
          <div
            className="fixed inset-0 bg-black/30 z-40 backdrop-blur-[2px]"
            aria-hidden
            onClick={() => setProfileOpen(false)}
          />
          <div className="fixed left-14 top-0 bottom-0 w-72 bg-identra-surface border-r border-identra-divider z-50 flex flex-col shadow-xl animate-slide-in-left">
            <div className="flex items-center justify-between px-4 py-3 border-b border-identra-border-subtle">
              <span className="text-xs font-semibold text-identra-text-secondary uppercase tracking-wider">Profile</span>
              <button
                onClick={() => setProfileOpen(false)}
                className="p-1.5 rounded-md text-identra-text-tertiary hover:text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                aria-label="Close"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-4 space-y-5">
              <div className="flex flex-col items-center">
                <div className="w-16 h-16 rounded-full bg-identra-surface-elevated border border-identra-border flex items-center justify-center mb-3">
                  <User className="w-8 h-8 text-identra-text-secondary" />
                </div>
                <p className="text-sm font-semibold text-identra-text-primary">User Profile</p>
                <p className="text-xs text-identra-text-tertiary mt-0.5">Active Session</p>
              </div>
              <div className="border-t border-identra-border-subtle pt-4">
                <p className="text-[10px] font-semibold text-identra-text-tertiary uppercase tracking-wider mb-2">Pack details</p>
                <div className="px-3 py-2.5 bg-identra-surface-elevated/80 border border-identra-border-subtle rounded-lg">
                  <p className="text-sm font-medium text-identra-text-primary">Standard Pack</p>
                  <p className="text-[10px] text-identra-text-tertiary mt-0.5">Current plan</p>
                </div>
              </div>
            </div>
          </div>
        </>
      )}

      {/* Settings panel */}
      {settingsOpen && (
        <>
          <div
            className="fixed inset-0 bg-black/30 z-40 backdrop-blur-[2px]"
            aria-hidden
            onClick={() => setSettingsOpen(false)}
          />
          <div className="fixed left-14 top-0 bottom-0 w-72 bg-identra-surface border-r border-identra-divider z-50 flex flex-col shadow-xl animate-slide-in-left">
            <div className="flex items-center justify-between px-4 py-3 border-b border-identra-border-subtle">
              <span className="text-xs font-semibold text-identra-text-secondary uppercase tracking-wider">Settings</span>
              <button
                onClick={() => setSettingsOpen(false)}
                className="p-1.5 rounded-md text-identra-text-tertiary hover:text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                aria-label="Close"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-4">
              <ul className="space-y-0.5">
                <li>
                  <button
                    onClick={() => setThemeOpen((v) => !v)}
                    className="w-full flex items-center justify-between gap-3 px-3 py-2.5 rounded-lg text-left text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <Moon className="w-4 h-4 text-identra-text-tertiary" />
                      <span>Theme</span>
                    </div>
                    <span className="text-xs text-identra-text-tertiary capitalize">{theme === "grey" ? "Grey" : theme}</span>
                    <ChevronDown className={`w-4 h-4 text-identra-text-tertiary transition-transform ${themeOpen ? "rotate-180" : ""}`} />
                  </button>
                  {themeOpen && (
                    <div className="mt-1 ml-4 pl-3 border-l border-identra-border-subtle space-y-0.5">
                      <button
                        onClick={() => setTheme("dark")}
                        className="w-full flex items-center justify-between gap-2 px-2 py-2 rounded-md text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                      >
                        <div className="flex items-center gap-2">
                          <Moon className="w-4 h-4 text-identra-text-tertiary" />
                          <span>Dark</span>
                        </div>
                        {theme === "dark" && <Check className="w-4 h-4 text-identra-primary shrink-0" />}
                      </button>
                      <button
                        onClick={() => setTheme("grey")}
                        className="w-full flex items-center justify-between gap-2 px-2 py-2 rounded-md text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                      >
                        <div className="flex items-center gap-2">
                          <Circle className="w-4 h-4 text-identra-text-tertiary" />
                          <span>Grey</span>
                        </div>
                        {theme === "grey" && <Check className="w-4 h-4 text-identra-primary shrink-0" />}
                      </button>
                      <button
                        onClick={() => setTheme("light")}
                        className="w-full flex items-center justify-between gap-2 px-2 py-2 rounded-md text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors"
                      >
                        <div className="flex items-center gap-2">
                          <Sun className="w-4 h-4 text-identra-text-tertiary" />
                          <span>Light</span>
                        </div>
                        {theme === "light" && <Check className="w-4 h-4 text-identra-primary shrink-0" />}
                      </button>
                    </div>
                  )}
                </li>
                <li>
                  <button className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors">
                    <Bell className="w-4 h-4 text-identra-text-tertiary" />
                    <span>Notifications</span>
                  </button>
                </li>
                <li>
                  <button className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors">
                    <Shield className="w-4 h-4 text-identra-text-tertiary" />
                    <span>Privacy & Security</span>
                  </button>
                </li>
                <li>
                  <button className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left text-sm text-identra-text-primary hover:bg-identra-surface-elevated transition-colors">
                    <LogOut className="w-4 h-4 text-identra-text-tertiary" />
                    <span>Account</span>
                  </button>
                </li>
              </ul>
            </div>
          </div>
        </>
      )}

      {/* Middle Section - Chat */}
      <main className="flex-1 flex flex-col min-w-0 border-r border-identra-divider">
        {/* Messages Area */}
        <div className="flex-1 overflow-y-auto px-8 py-8">
          {messages.length === 0 ? (
            <div className="flex flex-col h-full items-center justify-center px-4">
              <div className="w-full max-w-2xl">
                {/* IDENTRA + chat blended into background — no box */}
                <div className="mb-5 text-center">
                  <h2 className="text-2xl font-bold tracking-tight text-identra-text-primary drop-shadow-[0_0_16px_rgba(120,119,198,0.5)]">
                    IDENTRA
                  </h2>
                </div>
                <div className="space-y-4">
                  <div className="relative">
                    <textarea
                      ref={textareaRef}
                      value={input}
                      onChange={(e) => setInput(e.target.value)}
                      onKeyDown={handleKeyPress}
                      placeholder="Type a message to Identra to get started..."
                      className="w-full bg-identra-surface/60 border border-identra-border-subtle focus:border-identra-primary/70 rounded-lg px-4 py-4 pr-12 text-sm text-identra-text-primary placeholder:text-identra-text-tertiary outline-none transition-all duration-75 focus:bg-identra-surface/80 backdrop-blur-sm resize-none overflow-y-auto"
                      disabled={isProcessing}
                      style={{ minHeight: '80px', maxHeight: '400px' }}
                    />
                    <button
                      onClick={handleSend}
                      disabled={!input.trim() || isProcessing}
                      className="absolute right-3 bottom-3 p-2 text-identra-text-tertiary hover:text-identra-text-primary disabled:text-identra-text-disabled hover:bg-identra-surface/80 rounded transition-all duration-75"
                    >
                      <Send className="w-5 h-5" />
                    </button>
                  </div>
                  <div className="flex items-center gap-3 justify-end">
                    <span className="text-[10px] font-semibold text-identra-text-tertiary/80 uppercase tracking-[0.16em] shrink-0">
                      {currentModel?.name}
                    </span>
                    <div className="flex items-center gap-2">
                      {models.map((model) => (
                        <button
                          key={model.id}
                          onClick={() => setSelectedModel(model.id)}
                          className={`flex items-center justify-center w-8 h-8 rounded-full border transition-all duration-100 p-1.5 ${
                            selectedModel === model.id
                              ? 'border-identra-primary/80 bg-identra-surface/80 shadow-[0_0_10px_rgba(120,119,198,0.5)]'
                              : 'border-identra-border-subtle bg-identra-surface/60 hover:border-identra-primary/60'
                          }`}
                          title={model.name}
                        >
                          {(() => {
                          const Icon = modelIcons[model.id];
                          return Icon ? <Icon className="w-full h-full text-identra-text-secondary" /> : null;
                        })()}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="space-y-6 max-w-full">
              {messages.map((msg) => (
                <div 
                  key={msg.id} 
                  className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div className="w-[60%] flex flex-col gap-2">
                    <div className={`px-4 py-3 rounded-lg ${
                      msg.role === 'user' 
                        ? 'bg-identra-surface-elevated border border-identra-border text-identra-text-primary ml-auto' 
                        : 'bg-identra-surface border border-identra-border-subtle text-identra-text-primary mr-auto'
                    }`}>
                      <p className="text-sm leading-relaxed whitespace-pre-wrap">{msg.content}</p>
                    </div>
                    <div className={`flex items-center gap-2 ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                      {msg.model && (
                        <span className="text-[9px] text-identra-text-muted uppercase tracking-wider">
                          {models.find(m => m.id === msg.model)?.name}
                        </span>
                      )}
                      <span className="text-[9px] text-identra-text-muted">
                        {msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
              
              {isProcessing && (
                <div className="flex justify-start">
                  <div className="w-[60%] flex flex-col gap-2">
                    <div className="px-4 py-3 bg-identra-surface border border-identra-border-subtle rounded-lg mr-auto">
                      <div className="flex gap-1">
                        <span className="w-1.5 h-1.5 bg-identra-text-muted rounded-full animate-pulse"></span>
                        <span className="w-1.5 h-1.5 bg-identra-text-muted rounded-full animate-pulse" style={{ animationDelay: '0.2s' }}></span>
                        <span className="w-1.5 h-1.5 bg-identra-text-muted rounded-full animate-pulse" style={{ animationDelay: '0.4s' }}></span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
              <div ref={messagesEndRef} />

              {/* Chat input blended into background — no box */}
              <div className="max-w-5xl mx-auto pt-6 px-4">
                <div className="mb-4 text-center">
                  <h2 className="text-xl font-bold tracking-tight text-identra-text-primary drop-shadow-[0_0_12px_rgba(120,119,198,0.5)]">
                    IDENTRA
                  </h2>
                </div>
                <div className="space-y-4">
                  <div className="relative">
                    <textarea
                      ref={textareaRef}
                      value={input}
                      onChange={(e) => setInput(e.target.value)}
                      onKeyDown={handleKeyPress}
                      placeholder="Type a message to Identra..."
                      className="w-full bg-identra-surface/60 border border-identra-border-subtle focus:border-identra-primary/70 rounded-lg px-4 py-4 pr-12 text-sm text-identra-text-primary placeholder:text-identra-text-tertiary outline-none transition-all duration-75 focus:bg-identra-surface/80 backdrop-blur-sm resize-none overflow-y-auto"
                      disabled={isProcessing}
                      style={{ minHeight: '80px', maxHeight: '400px' }}
                    />
                    <button
                      onClick={handleSend}
                      disabled={!input.trim() || isProcessing}
                      className="absolute right-3 bottom-3 p-2 text-identra-text-tertiary hover:text-identra-text-primary disabled:text-identra-text-disabled hover:bg-identra-surface/80 rounded transition-all duration-75"
                    >
                      <Send className="w-5 h-5" />
                    </button>
                  </div>
                  <div className="flex items-center gap-3 justify-end">
                    <span className="text-[10px] font-semibold text-identra-text-tertiary/80 uppercase tracking-[0.16em] shrink-0">
                      {currentModel?.name}
                    </span>
                    <div className="flex items-center gap-2">
                      {models.map((model) => (
                        <button
                          key={model.id}
                          onClick={() => setSelectedModel(model.id)}
                          className={`flex items-center justify-center w-8 h-8 rounded-full border transition-all duration-100 p-1.5 ${
                            selectedModel === model.id
                              ? 'border-identra-primary/80 bg-identra-surface/80 shadow-[0_0_10px_rgba(120,119,198,0.5)]'
                              : 'border-identra-border-subtle bg-identra-surface/60 hover:border-identra-primary/60'
                          }`}
                          title={model.name}
                        >
                          {(() => {
                          const Icon = modelIcons[model.id];
                          return Icon ? <Icon className="w-full h-full text-identra-text-secondary" /> : null;
                        })()}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </main>

      {/* Right Section - Model Context and Recent Chats */}
      <aside className="w-72 bg-identra-surface border-l border-identra-divider flex flex-col">
        {/* Model Context and Audits */}
        <div className="px-4 py-5 border-b border-identra-border-subtle">
          <h3 className="text-[10px] font-semibold text-identra-text-secondary uppercase tracking-[0.1em] mb-4">
            Model Context
          </h3>
          <div className="space-y-2.5">
            {contextDocuments.map((doc) => {
              const docModel = models.find(m => m.id === doc.model);
              return (
                <div 
                  key={doc.id}
                  className="px-3 py-2.5 bg-identra-surface-elevated border border-identra-border hover:border-identra-primary transition-all duration-75 cursor-pointer group rounded"
                >
                  <div className="flex items-start gap-2.5 mb-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-identra-active shrink-0 mt-1.5"></div>
                    <div className="flex-1 min-w-0">
                      <p className="text-xs text-identra-text-primary font-medium truncate group-hover:text-identra-text-primary">
                        {doc.name}
                      </p>
                      <p className="text-[10px] text-identra-text-muted mt-1">
                        {doc.size}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center justify-between border-t border-identra-border-subtle pt-2 mt-2">
                    <span className="text-[10px] text-identra-text-tertiary uppercase tracking-wider font-medium">
                      {docModel.name}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>

        </div>

        {/* Recent Chats */}
        <div className="flex-1 overflow-y-auto px-4 py-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-[10px] font-semibold text-identra-text-secondary uppercase tracking-[0.1em]">
              Recent Chats
            </h3>
            <Search className="w-3.5 h-3.5 text-identra-text-tertiary" />
          </div>
          <div className="space-y-2">
            {conversationHistory.length === 0 ? (
              <div className="px-3 py-6 text-center bg-identra-surface-elevated border border-identra-border rounded">
                <p className="text-[10px] text-identra-text-muted">No conversations yet</p>
              </div>
            ) : (
              conversationHistory.map((item) => {
                const timestamp = new Date(item.timestamp * 1000);
                const timeAgo = Math.floor((Date.now() - timestamp) / 1000 / 60);
                let timeStr;
                if (timeAgo < 60) {
                  timeStr = `${timeAgo}m ago`;
                } else if (timeAgo < 1440) {
                  timeStr = `${Math.floor(timeAgo / 60)}h ago`;
                } else {
                  timeStr = `${Math.floor(timeAgo / 1440)}d ago`;
                }
                
                // Try to parse and show user message as preview
                let title = "Conversation";
                
                return (
                  <button
                    key={item.id} 
                    onClick={() => handleLoadConversation(item)}
                    className="w-full px-3 py-3 bg-identra-surface-elevated border border-identra-border hover:border-identra-primary cursor-pointer transition-all duration-75 group rounded text-left"
                  >
                    <div className="flex items-center gap-2.5 mb-2">
                      <FileText className="w-3.5 h-3.5 text-identra-text-tertiary group-hover:text-identra-text-secondary" />
                      <p className="text-xs text-identra-text-secondary group-hover:text-identra-text-primary font-medium line-clamp-2 flex-1">
                        {title}
                      </p>
                    </div>
                    <p className="text-[10px] text-identra-text-muted pl-6">{timeStr}</p>
                  </button>
                );
              })
            )}
          </div>
        </div>

        {/* Footer Status */}
        <div className="px-4 py-3 border-t border-identra-border-subtle">
          <div className="flex items-center justify-center gap-2">
            <div className="w-1.5 h-1.5 rounded-full bg-identra-active"></div>
            <div className="text-[10px] text-identra-text-tertiary text-center tracking-[0.1em] font-semibold">
              CROSS-MODEL SYNC ACTIVE
            </div>
          </div>
        </div>
      </aside>
    </div>
  );
}

