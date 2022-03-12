import React from "react";
import {Toast as BToast, ToastContainer as BToastContainer} from "react-bootstrap";

// ref. https://www.wantedly.com/users/26190108/post_articles/345447

type ToastType = "none" | "success" | "failure";

const ToastContext = React.createContext(({}: { text: string, type: ToastType }) => {
});
ToastContext.displayName = "ToastContext";

export const useToast = () => {
    return React.useContext(ToastContext);
}

function ToastSuccess({text, onClose}: { text: string, onClose: () => void }) {
    return (
        <BToastContainer position="top-end" className="p-3">
            <BToast bg="success" onClose={onClose} delay={3000} autohide>
                <BToast.Body className="text-white">{text}</BToast.Body>
            </BToast>
        </BToastContainer>
    )
}

function ToastFailure({text, onClose}: { text: string, onClose: () => void }) {
    return (
        <BToastContainer position="top-end" className="p-3">
            <BToast bg="danger" onClose={onClose} delay={3000} autohide>
                <BToast.Body className="text-white">{text}</BToast.Body>
            </BToast>
        </BToastContainer>
    )
}


export const ToastProvider: React.FC = ({children}) => {
    const [toastText, setToastText] = React.useState("");
    const [toastType, setToastType] = React.useState<ToastType>("none");

    const showToast = ({text, type}: { text: string, type: ToastType }) => {
        setToastText(text);
        setToastType(type);
    }

    return (
        <ToastContext.Provider value={showToast}>
            {children}

            {toastType === "success" ?
                <ToastSuccess text={toastText} onClose={() => setToastType('none')}/> : null}
            {toastType === "failure" ?
                <ToastFailure text={toastText} onClose={() => setToastType('none')}/> : null}
        </ToastContext.Provider>
    )
}