import React from 'react';
import Navbar from './Navbar';
import './App.css';
import {Routes, Route} from 'react-router-dom';
import Home from './Home';
import UpdateBlockchain from "./UpdateBlockchain";
import {Toast as BToast, ToastContainer as BToastContainer} from "react-bootstrap";

function UpdateBlockchainSuccessToast({onClose}: { onClose: () => void }) {
    return (
        <BToastContainer position="top-end" className="p-3">
            <BToast bg="success" onClose={onClose} delay={3000} autohide>
                <BToast.Body className="text-white">Blockchain updated successfully</BToast.Body>
            </BToast>
        </BToastContainer>
    )
}

function UpdateBlockchainFailureToast({onClose}: { onClose: () => void }) {
    return (
        <BToastContainer position="top-end" className="p-3">
            <BToast bg="danger" onClose={onClose} delay={3000} autohide>
                <BToast.Body className="text-white">Failed to update Blockchain</BToast.Body>
            </BToast>
        </BToastContainer>
    )
}

function App() {
    const [toastType, setToastType] = React.useState('none');

    const onUpdateBlockchainSuccess = () => {
        setToastType('update-blockchain-success');
    }

    const onUpdateBlockchainFailure = () => {
        setToastType('update-blockchain-failure');
    }

    return (
        <>
            <div className="App">
                <Navbar/>
            </div>
            <Routes>
                <Route path="/home" element={<Home/>}/>
                <Route path="/update-blockchain"
                       element={<UpdateBlockchain onUpdateBlockchainSuccess={onUpdateBlockchainSuccess}
                                                  onUpdateBlockchainFailure={onUpdateBlockchainFailure}/>}/>
            </Routes>

            {toastType === 'update-blockchain-success' ?
                <UpdateBlockchainSuccessToast onClose={() => setToastType('none')}/> : null}
            {toastType === 'update-blockchain-failure' ?
                <UpdateBlockchainFailureToast onClose={() => setToastType('none')}/> : null}
        </>
    );
}

export default App;
