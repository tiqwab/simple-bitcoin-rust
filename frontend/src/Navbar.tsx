import {Container as BContainer, Nav as BNav, Navbar as BNavbar} from "react-bootstrap";
import NavbarMenu from './NavbarMenu';
import NavbarSettings from './NavbarSettings';
import NavbarAdvance from './NavbarAdvance';
import { LinkContainer as BLinkContainer } from "react-router-bootstrap";

function Navbar() {
    return (
        <BNavbar bg="light" expand="lg">
            <BContainer>
                <BLinkContainer to="/home">
                    <BNavbar.Brand>Simple Bitcoin</BNavbar.Brand>
                </BLinkContainer>
                <BNavbar.Toggle aria-controls="basic-navbar-nav"/>
                <BNavbar.Collapse id="basic-navbar-nav">
                    <BNav className="me-auto">
                        <NavbarMenu/>
                        <NavbarSettings/>
                        <NavbarAdvance/>
                    </BNav>
                </BNavbar.Collapse>
            </BContainer>
        </BNavbar>
    )
}

export default Navbar;