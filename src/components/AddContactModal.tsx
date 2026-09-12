import React, { useState, useMemo } from "react";
import {
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Button,
  Input,
  Field,
} from "@fluentui/react-components";
import { Dismiss24Regular, PersonAdd20Regular } from "@fluentui/react-icons";

interface AddContactModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAdd: (peerIdHex: string, name: string) => Promise<void>;
}

export const AddContactModal: React.FC<AddContactModalProps> = ({
  isOpen,
  onClose,
  onAdd,
}) => {
  const [name, setName] = useState("");
  const [peerId, setPeerId] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const cleanHex = useMemo(() => {
    return peerId.trim().replace(/^0x/i, "");
  }, [peerId]);

  const isValidHex = useMemo(() => {
    return /^[0-9a-fA-F]{64}$/.test(cleanHex);
  }, [cleanHex]);

  const validationState = useMemo(() => {
    if (!peerId) return "none";
    return isValidHex ? "success" : "error";
  }, [peerId, isValidHex]);

  const validationMessage = useMemo(() => {
    if (!peerId) return "Введите 64-символьный hex-ключ пира";
    if (cleanHex.length !== 64) {
      return `Длина ключа: ${cleanHex.length}/64 символов`;
    }
    if (!/^[0-9a-fA-F]+$/.test(cleanHex)) {
      return "Ключ должен содержать только шестнадцатеричные символы (0-9, a-f)";
    }
    return "Корректный 32-байтный публичный ключ";
  }, [peerId, cleanHex]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValidHex || !name.trim()) return;

    setIsSubmitting(true);
    setErrorMessage(null);
    try {
      await onAdd(cleanHex, name.trim());
      setName("");
      setPeerId("");
      onClose();
    } catch (err: any) {
      setErrorMessage(err?.toString() || "Ошибка при добавлении контакта");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={(_, data) => !data.open && onClose()}>
      <DialogSurface style={{ maxWidth: "480px" }}>
        <form onSubmit={handleSubmit}>
          <DialogBody>
            <DialogTitle
              action={
                <Button
                  appearance="subtle"
                  aria-label="close"
                  icon={<Dismiss24Regular />}
                  onClick={onClose}
                />
              }
            >
              Добавить новый контакт
            </DialogTitle>
            <DialogContent style={{ display: "flex", flexDirection: "column", gap: "16px", marginTop: "8px" }}>
              <Field label="Имя пира / Метка" required>
                <Input
                  value={name}
                  onChange={(_, data) => setName(data.value)}
                  placeholder="Например: Alice (Laptop)"
                  disabled={isSubmitting}
                />
              </Field>

              <Field
                label="Публичный ключ пира (64 Hex)"
                required
                validationState={validationState}
                validationMessage={validationMessage}
              >
                <Input
                  value={peerId}
                  onChange={(_, data) => setPeerId(data.value)}
                  placeholder="32 байта в шестнадцатеричном формате..."
                  style={{ fontFamily: "monospace", fontSize: "12px" }}
                  disabled={isSubmitting}
                />
              </Field>

              {errorMessage && (
                <div style={{ color: "#ff5252", fontSize: "12px" }}>
                  {errorMessage}
                </div>
              )}
            </DialogContent>
            <DialogActions style={{ marginTop: "20px" }}>
              <Button appearance="secondary" onClick={onClose} disabled={isSubmitting}>
                Отмена
              </Button>
              <Button
                appearance="primary"
                type="submit"
                icon={<PersonAdd20Regular />}
                disabled={!isValidHex || !name.trim() || isSubmitting}
              >
                {isSubmitting ? "Добавление..." : "Добавить"}
              </Button>
            </DialogActions>
          </DialogBody>
        </form>
      </DialogSurface>
    </Dialog>
  );
};
