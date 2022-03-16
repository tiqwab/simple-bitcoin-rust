import {NavDropdown as BNavDropdown} from "react-bootstrap";

function NavbarAdvance() {
    return (
        <BNavDropdown title="Advance" id="navbar-advance">
            <BNavDropdown.Item disabled={true}>Show logs</BNavDropdown.Item>
            <BNavDropdown.Item disabled={true}>Show Blockchain</BNavDropdown.Item>
        </BNavDropdown>
    )
}

export default NavbarAdvance;
