import { Injectable } from "@angular/core";
import { HttpClient, HttpHeaders } from "@angular/common/http";
import { Observable, BehaviorSubject } from "rxjs";
import { map } from "rxjs/operators";
import { session } from "./model";

export enum ContactType {
  Email = 1,
  Phone
}

export interface Contact {
  id: number;
  identity: string;
  type_id: ContactType;
  status: number;
}

export interface ContactStore {
  contacts: Contact[];
}

@Injectable()
export class ContactModelService {
  private store: ContactStore;
  private subject: BehaviorSubject<Contact[]>;

  constructor(private http: HttpClient) {
    this.store = {
      contacts: []
    };
    this.subject = new BehaviorSubject<Contact[]>([]);
  }

  get contacts() {
    return this.subject.asObservable();
  }

  select(userId: number) {
    let headers = new HttpHeaders({
      "Content-Type": "application/json"
    });
    let options = {
      headers: headers
    };

    let apiUri = `/api/v1/users/${userId}/contacts`;
    this.http.get(apiUri, options).subscribe((contacts: Contact[]) => {
      this.store.contacts = contacts;
      this.subject.next(contacts);
    });
  }

  create(contact: Contact) {
    let headers = new HttpHeaders({
      "Content-Type": "application/json"
    });
    let options = {
      headers: headers
    };

    let apiUri = "/api/v1/users/" + session.currUser().id + "/contacts";
    return this.http.post(apiUri, contact, options).pipe(
      map((contact: Contact) => {
        this.store.contacts.push(contact);
        this.subject.next(Object.assign({}, this.store).contacts);

        return contact;
      })
    );
  }

  remove(contact: Contact) {
    let apiUri = `/api/v1/users/${session.currUser().id}/contacts/${
      contact.id
    }`;

    return this.http.delete(apiUri).pipe(
      map((contact: Contact) => {
        let index = this.store.contacts.findIndex(
          _contact => _contact.id === contact.id
        );
        this.store.contacts.splice(index, 1);

        this.subject.next(Object.assign({}, this.store).contacts);

        return contact;
      })
    );
  }
}
